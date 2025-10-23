use std::fmt::Debug;

pub struct ChainDebugAsList<T>(T);

impl<T> Debug for ChainDebugAsList<&T>
where
    T: ChainDebug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        self.0.fmt_as_list(&mut debug_list);
        debug_list.finish()
    }
}

pub trait ChainDebug {
    fn fmt_as_list(&self, f: &mut std::fmt::DebugList<'_, '_>);
    fn as_list(&self) -> ChainDebugAsList<&Self> {
        ChainDebugAsList(self)
    }
}

impl<Head, Tail> ChainDebug for (Head, Tail)
where
    Head: ChainDebug,
    Tail: Debug,
{
    fn fmt_as_list(&self, f: &mut std::fmt::DebugList<'_, '_>) {
        let (head, tail) = self;
        head.fmt_as_list(f);
        f.entry(tail);
    }
}

impl<Head> ChainDebug for (Head,)
where
    Head: Debug,
{
    fn fmt_as_list(&self, f: &mut std::fmt::DebugList<'_, '_>) {
        f.entry(&self.0);
    }
}
