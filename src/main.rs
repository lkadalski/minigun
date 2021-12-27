mod errors;
mod options;
mod report_generator;
mod test_dispatcher;
mod test_runner;

use crate::options::Options;
use crate::test_dispatcher::TestCommand;
use test_dispatcher::initialize;

fn main() {
    let test_command = TestCommand::new(Box::new(Options::new()));
    initialize(test_command);
}
