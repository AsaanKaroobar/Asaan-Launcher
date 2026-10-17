# Read-only private-session accessibility probe.
import gi,json,os
if 'asaaan-x11-' not in os.environ.get('XDG_RUNTIME_DIR',''):raise RuntimeError('Use isolated harness')
gi.require_version('Atspi','2.0')
from gi.repository import Atspi,GLib
result={'query':None,'selected':[],'text':[],'windows':[]}
def visit(node,depth=0):
 if node is None or depth>25:return
 try:
  name=node.get_name();role=node.get_role_name()
  if role in ['frame','window']:result['windows'].append(name)
  if name=='Pick a line':
   text=node.get_text_iface()
   if text:result['query']=Atspi.Text.get_text(text,0,-1)
  if node.get_state_set().contains(Atspi.StateType.SELECTED):result['selected'].append(name)
  if name:result['text'].append(name)
  text=node.get_text_iface()
  if text:result['text'].append(Atspi.Text.get_text(text,0,-1))
  for i in range(node.get_child_count()):visit(node.get_child_at_index(i),depth+1)
 except (GLib.Error,AttributeError):pass
visit(Atspi.get_desktop(0));print(json.dumps(result))
