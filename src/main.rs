mod options;
mod test_dispatcher;
mod errors;
mod test_runner;
mod report_generator;

use crate::options::Options;
use test_dispatcher::initialize;
use crate::test_dispatcher::TestCommand;

fn main() {
    let test_command = TestCommand::new( Box::new( Options::new()));
    initialize(test_command);
}
