#[macro_export]
macro_rules! message_chain {
    () => {
        $crate::message::MessageChain::default()
    };
    ($($msg:expr),+ $(,)?) => {
        {
            let mut builder = $crate::message::MessageChainBuilder::new();
            $(builder.push($msg);)*
            builder.build()
        }
    }
}
