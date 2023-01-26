#[macro_export]
macro_rules! control_flow {
    (break) => {
        std::ops::ControlFlow::Break(None)
    };
    ($expr:expr; break) => {{
        $expr;

        std::ops::ControlFlow::Break(None)
    }};
    (break $expr:expr) => {
        std::ops::ControlFlow::Break(Some($expr))
    };

    (continue) => {
        std::ops::ControlFlow::Continue(None)
    };
    ($expr:expr; continue) => {{
        $expr;

        std::ops::ControlFlow::Continue(None)
    }};
    (continue $expr:expr) => {
        std::ops::ControlFlow::Continue(Some($expr))
    };
}
