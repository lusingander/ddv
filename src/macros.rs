#[macro_export]
macro_rules! handle_user_events {
    ($user_events:ident => $($event:pat => $body:block)+) => {
        #[allow(unreachable_code)]
        for user_event in &$user_events {
            match user_event {
                $($event => $body)+
                _ => {
                    continue;
                }
            }
            break;
        }
    };
}
