# Private multi-window app: logs graceful requests and can refuse them.
import os,sys,json,gi
if 'asaaan-x11-' not in os.environ.get('XDG_RUNTIME_DIR',''):raise RuntimeError('Use private harness')
gi.require_version('Gtk','3.0');gi.require_version('GdkX11','3.0')
from gi.repository import Gtk,GdkX11
log,mode=sys.argv[1:3]
windows=[]
def close(window,event):
    with open(log,'a') as f:f.write(json.dumps({'event':'close','window':window.get_window().get_xid()})+'\n')
    if open(mode).read().strip()!='accept':return True
    windows.remove(window);window.destroy()
    if not windows:Gtk.main_quit()
    return True
for i in range(2):
    w=Gtk.Window(title='Process Probe '+str(i));w.set_wmclass('ProcessProbe','ProcessProbe')
    w.set_focus_on_map(False);w.set_default_size(300,140);w.move(50+i*330,50)
    w.add(Gtk.Label(label='Unsaved work fixture — close may be refused'))
    w.connect('delete-event',close);w.show_all();windows.append(w)
print(json.dumps([w.get_window().get_xid() for w in windows]),flush=True)
Gtk.main()
