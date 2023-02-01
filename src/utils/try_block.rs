macro_rules! try_block {
    ($block:expr) => {
        (|| $block)()
    };
}

pub(crate) use try_block;
