use crate::app::AppState;
use crate::socket::response::Response;
use crate::ui::widgets::logs::Log;
use log::debug;

pub fn handle_message_event(app: &mut AppState, json_data: String) {
    app.set_socket_activity();
    debug!("received message: {}", json_data);
    app.append_log(Log {
        json_payload: json_data.clone(),
    });

    match Response::new_from_json_string(&json_data) {
        Response::JoinReply(reply) => {
            app.user.online_at = reply.user.online_at;
        }
        Response::PresenceDiff(diff) => {
            for user in diff.joins {
                let message = format!("@{} has joined #{}", user.username, app.room);
                app.add_user(user);
                app.add_system_message(message);
            }
            for user in diff.leaves {
                let message = format!("{} has left {}", user.username, app.room);
                app.remove_user(user);
                app.add_system_message(message);
            }
        }
        Response::PresenceState(state) => {
            app.set_users(state.users);
        }
        Response::RoomsUpdate(rooms) => {
            app.set_rooms(rooms);
        }
        Response::Shout(shout) => {
            if !shout.user.uuid.eq(&app.user.uuid) {
                let message = format!("{}: {}", shout.user.username, shout.message);
                app.add_user_message(message);
            }
        }
        Response::Unknown => (),
    }
}
