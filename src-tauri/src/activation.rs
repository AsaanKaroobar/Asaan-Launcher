//! X11 launchers represent direct user actions in EWMH activation requests.
#[cfg(target_os = "linux")]
pub fn activate(window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    use gtk::prelude::*;
    use x11rb::{
        connection::Connection,
        protocol::{
            Event,
            xproto::{
                AtomEnum, ChangeWindowAttributesAux, ClientMessageEvent, ConnectionExt, EventMask,
                PropMode,
            },
        },
        wrapper::ConnectionExt as _,
    };
    let widget = window.gtk_window()?;
    // Tauri dispatches show asynchronously; realize/map before reading the XID
    // and flush GTK's connection before sending the activation on another one.
    widget.realize();
    widget.show_all();
    widget.display().flush();
    let native = widget.window().ok_or("GTK window is not realized")?;
    let Ok(native) = native.downcast::<gdkx11::X11Window>() else {
        return Ok(());
    };
    let id = u32::try_from(native.xid())?;
    let (connection, screen) = x11rb::connect(None)?;
    let root = connection.setup().roots[screen].root;
    let active = connection
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
        .reply()?
        .atom;
    let clock = connection
        .intern_atom(false, b"_ASAAAN_ACTIVATION_TIME")?
        .reply()?
        .atom;
    // Obtain a server timestamp without borrowing another application's user-time property.
    // Event selection is per connection and does not replace GTK's subscription.
    connection
        .change_window_attributes(
            id,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )?
        .check()?;
    connection
        .change_property32(PropMode::REPLACE, id, clock, AtomEnum::CARDINAL, &[0])?
        .check()?;
    connection.flush()?;
    let timestamp = loop {
        if let Event::PropertyNotify(event) = connection.wait_for_event()?
            && event.window == id
            && event.atom == clock
        {
            break event.time;
        }
    };
    let request = ClientMessageEvent::new(32, id, active, [2, timestamp, 0, 0, 0]);
    connection
        .send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            request,
        )?
        .check()?;
    connection.flush()?;
    Ok(())
}
#[cfg(not(target_os = "linux"))]
pub fn activate(_window: &tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
