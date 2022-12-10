/// Functon that lauch the server and regulates all logic
pub mod run;
/// Trait that implement server-client communication as json files
pub mod protocol;
/// Regulates types of request that server can receive
pub mod request;
/// Rules for server response
pub mod response;
