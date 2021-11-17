mod options;
mod test_dispatcher;
mod errors;
mod test_runner;
mod report_generator;

use crate::options::Options;
use test_dispatcher::initialize;
use crate::test_dispatcher::TestCommand;

fn main() {
    //TODO add TestCommand::new().unwrap_or_else to throw validation errors
    // Provie test scenarios:
    // MultipleRequests * Multiple Connections (SHOOT"EM ALL)
    // -> Variations OneRequest x MultipleConnections
    //                     ManyRequest x OneConnection
    //                Request Rate
    //                File Upload
    let test_command = TestCommand::new( Box::new( Options::new()));
    initialize(test_command);
}
