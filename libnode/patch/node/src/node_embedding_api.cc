#include <memory>
#include <mutex>
#include <optional>
#include <vector>

#include "node_embedding_api.h"

#include "node.h"
#include "node_api.h"

#include "uv.h"

#include "v8.h"

namespace {
std::mutex env_mutex;
node::Environment* env_ptr = nullptr;

void set_env(node::Environment* env) {
  std::lock_guard<std::mutex> guard(env_mutex);
  env_ptr = env;
}

char* join_errors(const std::vector<std::string>& errors) {
  std::string joined_error;
  for (std::size_t i = 0; i < errors.size(); ++i) {
    if (i > 0) {
      joined_error += '\n';
    }
    joined_error += errors[i];
  }
  char* c_result = (char*)malloc(joined_error.size() + 1);
  joined_error.copy(c_result, joined_error.size());
  c_result[joined_error.size()] = '\0';
  return c_result;
}

std::vector<std::string> create_arg_vec(int argc, const char* const* argv) {
  std::vector<std::string> vec;
  if (argc > 0) {
    vec.reserve(argc);
    for (int i = 0; i < argc; ++i) {
      vec.emplace_back(argv[i]);
    }
  }
  return vec;
}

node_run_result_t RunNodeInstance(node::MultiIsolatePlatform* platform,
                                  const std::vector<std::string>& args,
                                  const std::vector<std::string>& exec_args,
                                  napi_addon_register_func napi_reg_func) {
  std::vector<std::string> errors;
  std::unique_ptr<node::CommonEnvironmentSetup> setup =
      node::CommonEnvironmentSetup::Create(platform, &errors, args, exec_args);

  if (!setup) {
    return {1, join_errors(errors)};
  }

  v8::Isolate* isolate = setup->isolate();
  node::Environment* env = setup->env();

  node_run_result_t result{0, nullptr};
  node::SetProcessExitHandler(env, [&](node::Environment* env, int exit_code) {
    result.exit_code = exit_code;
  });

  {
    v8::Locker locker(isolate);
    v8::Isolate::Scope isolate_scope(isolate);
    v8::HandleScope handle_scope(isolate);
    v8::Context::Scope context_scope(setup->context());

    node::AddLinkedBinding(env,
                           napi_module{
                               NAPI_MODULE_VERSION,
                               node::ModuleFlags::kLinked,
                               nullptr,
                               napi_reg_func,
                               "__embedder_mod",
                               nullptr,
                               {0},
                           });

    v8::MaybeLocal<v8::Value> loadenv_ret = node::LoadEnvironment(
        env,
        "const publicRequire = require('module').createRequire(process.cwd() "
        "+ '/');"
        "globalThis.require = publicRequire;"
        "globalThis.embedVars = { nön_ascıı: '🏳️‍🌈' };"
        "process._linkedBinding('__embedder_mod');");

    if (loadenv_ret.IsEmpty()) {
      result.exit_code = 1;
    }

    set_env(env);
    result.exit_code = node::SpinEventLoop(env).FromMaybe(0);
    set_env(nullptr);
  }

  node::Stop(env);

  return result;
}
}  // namespace

extern "C" {
node_run_result_t node_run(node_options_t options) {
  char** argv =
      uv_setup_args(options.process_argc, (char**)options.process_argv);
  std::vector<std::string> args(argv, argv + options.process_argc);
  std::unique_ptr<node::InitializationResult> result =
      node::InitializeOncePerProcess(
          args,
          {node::ProcessInitializationFlags::kNoInitializeV8,
           node::ProcessInitializationFlags::kNoInitializeNodeV8Platform});

  if (result->early_return() != 0) {
    return {result->exit_code(), join_errors(result->errors())};
  }

  std::unique_ptr<node::MultiIsolatePlatform> platform =
      node::MultiIsolatePlatform::Create(4);
  v8::V8::InitializePlatform(platform.get());
  v8::V8::Initialize();

  node_run_result_t ret =
      RunNodeInstance(platform.get(),
                      result->args(),
                      result->exec_args(),
                      napi_addon_register_func(options.napi_reg_func));

  v8::V8::Dispose();
  v8::V8::DisposePlatform();

  node::TearDownOncePerProcess();

  return ret;
}

int node_stop() {
  std::lock_guard<std::mutex> guard(env_mutex);
  if (env_ptr == nullptr) {
    return -1;
  }

  return node::Stop(env_ptr);
}
}
