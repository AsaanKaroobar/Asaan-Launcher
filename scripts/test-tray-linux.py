# Native KDE/StatusNotifier tray test adapter. Always targets one explicit PID.
import json, sys
from gi.repository import Gio, GLib
bus=Gio.bus_get_sync(Gio.BusType.SESSION,None)
def call(dest,path,interface,method,args=None):
 return bus.call_sync(dest,path,interface,method,args,None,Gio.DBusCallFlags.NONE,5000,None).unpack()
def unwrap(v):
 if isinstance(v,GLib.Variant):return unwrap(v.unpack())
 if isinstance(v,dict):return {k:unwrap(x) for k,x in v.items()}
 if isinstance(v,(list,tuple)):return [unwrap(x) for x in v]
 return v
items=unwrap(call('org.kde.StatusNotifierWatcher','/StatusNotifierWatcher','org.freedesktop.DBus.Properties','Get',GLib.Variant('(ss)',('org.kde.StatusNotifierWatcher','RegisteredStatusNotifierItems'))))[0]
selected=None
for item in items:
 name,tail=item.split('/',1)
 try:pid=call('org.freedesktop.DBus','/org/freedesktop/DBus','org.freedesktop.DBus','GetConnectionUnixProcessID',GLib.Variant('(s)',(name,)))[0]
 except GLib.Error:continue
 if pid==int(sys.argv[1]):selected=(name,'/'+tail);break
if not selected:raise RuntimeError('No native tray registered for requested PID')
name,path=selected
properties=unwrap(call(name,path,'org.freedesktop.DBus.Properties','GetAll',GLib.Variant('(s)',('org.kde.StatusNotifierItem',))))[0]
menu=properties['Menu']
layout=unwrap(call(name,menu,'com.canonical.dbusmenu','GetLayout',GLib.Variant('(iias)',(0,-1,[]))))
def rows(node):
 result=[{'id':node[0],**node[1]}]
 for child in node[2]:result+=rows(child)
 return result
entries=rows(layout[1])
labels=[row.get('label') for row in entries if row.get('label')]
expected=['Show Asaaan Launcher','Hide launcher','Quit Asaaan Launcher']
if not all(label in labels for label in expected):raise RuntimeError('Missing expected native tray actions: '+str(labels))
action=sys.argv[2] if len(sys.argv)>2 else 'inspect'
if action!='inspect':
 label={'show':expected[0],'hide':expected[1],'quit':expected[2]}[action]
 entry=next(row for row in entries if row.get('label')==label)
 call(name,menu,'com.canonical.dbusmenu','Event',GLib.Variant('(isvu)',(entry['id'],'clicked',GLib.Variant('i',0),0)))
print(json.dumps({'pid':int(sys.argv[1]),'action':action,'labels':labels}))
