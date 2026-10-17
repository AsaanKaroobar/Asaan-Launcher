# Private-session AT-SPI observer and accessibility actions for one explicit application PID.
import gi,json,os,sys
if 'asaaan-x11-' not in os.environ.get('XDG_RUNTIME_DIR',''):raise RuntimeError('Use isolated harness')
gi.require_version('Atspi','2.0')
from gi.repository import Atspi,GLib
mode=sys.argv[1];pid=int(sys.argv[2]);nodes=[]
def visit(node,depth=0):
 if node is None or depth>25:return
 try:
  name=node.get_name();role=node.get_role_name();text=node.get_text_iface();value=Atspi.Text.get_text(text,0,-1) if text else None
  nodes.append((node,{'name':name,'role':role,'text':value,'checked':node.get_state_set().contains(Atspi.StateType.CHECKED),'editable':node.get_state_set().contains(Atspi.StateType.EDITABLE),'enabled':node.get_state_set().contains(Atspi.StateType.ENABLED)}))
  for i in range(node.get_child_count()):visit(node.get_child_at_index(i),depth+1)
 except (GLib.Error,AttributeError):pass
root=Atspi.get_desktop(0)
for i in range(root.get_child_count()):
 app=root.get_child_at_index(i)
 if app and app.get_process_id()==pid:visit(app)
if mode=='state':print(json.dumps([value for node,value in nodes]))
else:
 name=sys.argv[3];context=sys.argv[4] if len(sys.argv)>4 else None;found=False
 for node,value in nodes:
  if value['name']!=name and not value['name'].startswith(name):continue
  if mode=='focus' and value['editable']:
   Atspi.Component.grab_focus(node.get_component_iface());found=True;break
  if mode=='click':
   if context:
    parent=node.get_parent();matches=False
    for depth in range(25):
     if parent is None:break
     if context in parent.get_name():matches=True;break
     parent=parent.get_parent()
    if not matches:continue
   action=node.get_action_iface()
   if action and Atspi.Action.get_n_actions(action)>0:Atspi.Action.do_action(action,0);found=True;break
 if not found:raise RuntimeError('Accessible action not found: '+name+' '+json.dumps([v for n,v in nodes]))
