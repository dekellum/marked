/// Compose a new filter closure, by chaining a list of closures or function
/// paths. Each is executed in order, while the return action remains
/// `Continue`.
#[macro_export]
macro_rules! chain_filters {
    ($first:expr $(, $subs:expr)* $(,)?) => (
        |node| {
            let mut action = $first(node);
        $(
            if action == $crate::vdom::filter::Action::Continue {
                action = $subs(node);
            }
        )*
            action
        }
    );
}
