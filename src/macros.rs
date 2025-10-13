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

#[macro_export]
macro_rules! handle_user_events_with_default {
    ($user_events:ident => $($event:pat => $body:block)+ => $default_block:block) => {
        for user_event in &$user_events {
            match user_event {
                $($event => $body)+
                _ => {
                    continue;
                }
            }
            return;
        }
        $default_block
    };
}
