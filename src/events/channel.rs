use crate::app::log::Log;
use crate::app::{AppState, SocketStatus};
use crate::socket::client::SocketEvent;
use crate::socket::response::Response;

// TODO: move response handling
// fn handle_socket_response(app: &mut AppState, json_data: String) {
// }

pub fn handle_socket_event(app: &mut AppState, socket_event: SocketEvent) {
    app.set_socket_activity();

    match socket_event {
        SocketEvent::Close => app.socket_status = SocketStatus::Closed,
        SocketEvent::Connect => app.socket_status = SocketStatus::Connected,
        SocketEvent::ConnectFail => app.socket_status = SocketStatus::ConnectFailed,
        SocketEvent::Disconnect => app.socket_status = SocketStatus::Disconnected,
        SocketEvent::Response(response) => {
            match response {
                Response::Unknown => (),
                _ => app.append_log(Log::new(response.clone())),
            }

            match response {
                Response::JoinReply(reply) => {
                    app.user.online_at = reply.user.online_at;
                }
                Response::PresenceDiff(diff) => {
                    for user in diff.joins {
                        let message = format!("@{} has joined #{}", user.username, app.room);
                        app.add_user(user);
                        app.add_system_public_message(message);
                    }
                    for user in diff.leaves {
                        let message = format!("@{} has left {}", user.username, app.room);
                        app.remove_user(user);
                        app.add_system_public_message(message);
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
                        app.add_user_message(shout.user, shout.message);
                    }
                }
                Response::Unknown => (),
            }
        }
    }
}
