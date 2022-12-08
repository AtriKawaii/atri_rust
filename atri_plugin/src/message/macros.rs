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

// experimental
macro_rules! _forward_message {
    () => {
        {
            $crate::message::forward::ForwardMessage::new()
        }
    };
    (
        builder = $builder:expr;
        {
            name: $name:expr,
            id: $id:expr,
            time: $time:expr,
            says: $msg:expr $(,)?
        }
    ) => {
        $builder.push_node(
            $crate::message::forward::ForwardNodeInfo {
                sender_id: $id,
                sender_name: $name.into(),
                time: $time
            },
            $msg
        );
    };
    (
        builder = $builder:expr;
        {
            name: $name:expr,
            id: $id:expr,
            says: $msg:expr $(,)?
        }
    ) => {
        $crate::forward_message {
            builder = $builder;
            {
                name: $name,
                id: $id,
                time: ::std::time::UNIX_EPOCH.elapsed().unwrap().as_secs(),
                says: $msg,
            }
        };
    };
    ($({ $tt:tt }),+ $(,)?) => {
        {
            let mut forward = $crate::message::forward::ForwardMessage::new();
            $crate::forward_message![
                $(
                builder: forward;
                {
                    $tt
                }
                )+
            ]

        }
    }
}
