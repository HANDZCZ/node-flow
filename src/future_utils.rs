#[derive(Debug)]
pub enum SoftFailPoll<T> {
    Pending,
    Ready(T),
    SoftFail,
}
