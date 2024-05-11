use crate::error::NodeError;

#[derive(Debug, Clone)]
pub struct NodeArgs {
    pub(crate) args: Vec<String>,
    pub(crate) insert_default_process_arg: bool,
}

impl NodeArgs {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            insert_default_process_arg: true,
        }
    }

    pub fn args<T, I>(mut self, args: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: ToString,
    {
        self.args = args.into_iter().map(|arg| arg.to_string()).collect();
        self
    }

    pub fn insert_default_process_arg(mut self, insert_default_process_arg: bool) -> Self {
        self.insert_default_process_arg = insert_default_process_arg;
        self
    }

    pub(crate) fn get_args(&self) -> crate::Result<Vec<String>> {
        let first_arg = std::env::args()
            .next()
            .ok_or_else(|| NodeError::generic("Failed to get the first run argument"))?;

        let mut args = self.args.clone();
        if (args.first().is_none() || args.first().unwrap() != &first_arg)
            && self.insert_default_process_arg
        {
            args.insert(0, first_arg);
        }

        Ok(args)
    }
}

impl Default for NodeArgs {
    fn default() -> Self {
        Self::new()
    }
}
