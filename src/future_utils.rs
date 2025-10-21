pub enum SoftFailPoll<T> {
    Pending,
    Ready(T),
    SoftFail,
}
