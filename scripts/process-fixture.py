# Private multi-window app: graceful refusal, delayed/background exit and owned test cleanup.
import os,sys,json,gi,subprocess,time
if 'asaaan-x11-' not in os.environ.get('XDG_RUNTIME_DIR',''):raise RuntimeError('Use private harness')
gi.require_version('Gtk','3.0');gi.require_version('GdkX11','3.0')
from gi.repository import Gtk,GdkX11,GLib
log,mode=sys.argv[1:3]
auto=sys.argv[3] if len(sys.argv)>3 else None
windows=[]
def current():
    try:return open(mode).read().strip()
    except OSError:return 'refuse'
def close(window,event):
    with open(log,'a') as f:f.write(json.dumps({'event':'close','window':window.get_window().get_xid(),'pid':os.getpid()})+'\n')
    value=current()
    if value not in ['accept','background','delayed']:return True
    windows.remove(window);window.destroy()
    if not windows:
        if value=='accept':Gtk.main_quit()
        elif value=='delayed':GLib.timeout_add(1200,lambda:(Gtk.main_quit(),False)[1])
    return True
for i in range(2):
    w=Gtk.Window(title='Process Probe '+str(i));w.set_wmclass('ProcessProbe','ProcessProbe')
    w.set_focus_on_map(False);w.set_default_size(300,140);w.move(50+i*330,50)
    w.add(Gtk.Label(label='Unsaved work fixture — close may be refused'))
    w.connect('delete-event',close);w.show_all();windows.append(w)
ids=[w.get_window().get_xid() for w in windows]
if auto:
    for w in ids:subprocess.run(['xprop','-id',str(w),'-f','_GTK_APPLICATION_ID','8u','-set','_GTK_APPLICATION_ID',auto],check=True,stdout=subprocess.DEVNULL)
with open(log+'.starts','a') as f:f.write(json.dumps({'pid':os.getpid(),'windows':ids,'at':time.monotonic()})+'\n')
print(json.dumps(ids),flush=True)
def control():
    value=current()
    if value=='exit':Gtk.main_quit();return False
    if value=='hang':
        while True:time.sleep(1)
    return True
GLib.timeout_add(50,control)
Gtk.main()
