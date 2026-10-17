# Read-only AT-SPI observation for native input/focus assertions; no text injection.
import gi, json
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi, GLib
states = []
def visit(node, depth=0):
    if depth > 12:
        return
    try:
        if node.get_name() == 'Search applications':
            text = node.get_text_iface()
            states.append({'text': Atspi.Text.get_text(text, 0, -1) if text else None})
            return
        for i in range(node.get_child_count()):
            visit(node.get_child_at_index(i), depth + 1)
    except GLib.Error:
        pass  # A foreground may exit while accessibility enumerates it.
visit(Atspi.get_desktop(0))
print(json.dumps(states))
