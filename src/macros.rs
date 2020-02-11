/// Compose a new filter closure, by chaining a list of closures or function
/// paths. Each is executed in order, while the return action remains
/// `Continue`.
#[macro_export]
macro_rules! chain_filters {
    ($first:expr $(, $subs:expr)* $(,)?) => (
        |node: &mut $crate::Node| {
            let mut action: $crate::filter::Action = $first(node);
        $(
            if action == $crate::filter::Action::Continue {
                action = $subs(node);
            }
        )*
            action
        }
    );
}
