# Read-only AT-SPI snapshot for the native Wayland launcher.
import gi, json, sys
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi, GLib

pid = int(sys.argv[1])
result = {"search": None, "items": [], "controls": []}

def visit(node, depth=0):
    if node is None or depth > 25:
        return
    try:
        name = node.get_name()
        role = node.get_role_name()
        if name == 'Search applications':
            text = node.get_text_iface()
            result["search"] = Atspi.Text.get_text(text, 0, -1) if text else None
        if role == 'list item' and name:
            result["items"].append(name)
        if role in ('push button', 'combo box') and name:
            result["controls"].append(name)
        for index in range(node.get_child_count()):
            visit(node.get_child_at_index(index), depth + 1)
    except (GLib.Error, AttributeError):
        pass

root = Atspi.get_desktop(0)
for index in range(root.get_child_count()):
    app = root.get_child_at_index(index)
    if app and app.get_process_id() == pid:
        visit(app)
print(json.dumps(result))
