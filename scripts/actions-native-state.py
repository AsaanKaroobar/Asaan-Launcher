# Read-only native observations in the private X11 test session.
import os, sys, json, gi
if 'asaaan-x11-' not in os.environ.get('XDG_RUNTIME_DIR',''): raise RuntimeError('Use the isolated harness')
gi.require_version('Gtk','3.0'); gi.require_version('Atspi','2.0')
from gi.repository import Gtk, Gdk, Atspi, GLib
if sys.argv[1]=='clipboard':
    Gtk.init([]); print(json.dumps(Gtk.Clipboard.get(Gdk.SELECTION_CLIPBOARD).wait_for_text(),ensure_ascii=False)); sys.exit(0)
result={'query':None,'menu':[],'text':[],'selected':[],'windows':[]}
visited=0
def visit(node,depth=0):
    global visited
    if node is None:return
    visited+=1
    if depth>25 or visited>6000:return
    try:
        name=node.get_name(); role=node.get_role_name()
        if name=='Search applications':
            result['query']=Atspi.Text.get_text(node.get_text_iface(),0,-1)
        if role=='menu item':result['menu'].append(name)
        if role in ['frame','window']:result['windows'].append(name)
        if name and ('Copied' in name or 'File manager' in name or 'removed' in name or 'catalog' in name or any(t in name for t in ['Quitting','Hide minimizes','identity','executable','remote','no longer running','instance changed'])):result['text'].append(name)
        text=node.get_text_iface()
        if text:
            value=Atspi.Text.get_text(text,0,-1)
            if value and any(token in value for token in ['Copied','File manager','removed','catalog','failed','Quitting','Hide minimizes','identity','executable','remote','no longer running','instance changed']):result['text'].append(value)
        if name and node.get_state_set().contains(Atspi.StateType.SELECTED):result['selected'].append(name)
        for i in range(node.get_child_count()):visit(node.get_child_at_index(i),depth+1)
    except (GLib.Error, AttributeError):pass
visit(Atspi.get_desktop(0)); print(json.dumps(result,ensure_ascii=False))
