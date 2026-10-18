# Read-only observation for the private launch/reuse harness.
import os, sys, json, gi
if 'asaaan-x11-' not in os.environ.get('XDG_RUNTIME_DIR', ''):
    raise RuntimeError('Use the isolated harness')
gi.require_version('Atspi', '2.0')
from gi.repository import Atspi, GLib
result = {'query': None, 'rows': []}
visited = 0
def visit(node, depth=0):
    global visited
    if node is None or depth > 25 or visited > 6000:
        return
    visited += 1
    try:
        name = node.get_name()
        if name == 'Search applications':
            result['query'] = Atspi.Text.get_text(node.get_text_iface(), 0, -1)
        if node.get_role_name() == 'list item':
            result['rows'].append(name)
        for i in range(node.get_child_count()):
            visit(node.get_child_at_index(i), depth + 1)
    except (GLib.Error, AttributeError):
        pass
root = Atspi.get_desktop(0)
for i in range(root.get_child_count()):
    app = root.get_child_at_index(i)
    if app and app.get_process_id() == int(sys.argv[1]):
        visit(app)
print(json.dumps(result, ensure_ascii=False))
