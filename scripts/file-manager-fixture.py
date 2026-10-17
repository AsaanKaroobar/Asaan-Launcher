# Test-only FileManager1 service: records literal URIs and simulates accept/refuse/hang.
import os, sys, json, gi
if 'asaaan-x11-' not in os.environ.get('XDG_RUNTIME_DIR',''): raise RuntimeError('Use the isolated harness')
from gi.repository import Gio, GLib
mode, marker=sys.argv[1:3]
xml='<node><interface name="org.freedesktop.FileManager1"><method name="ShowItems"><arg type="as" direction="in"/><arg type="s" direction="in"/></method></interface></node>'
info=Gio.DBusNodeInfo.new_for_xml(xml).interfaces[0]
def call(connection,sender,path,interface,method,parameters,invocation):
    uris,startup=parameters.unpack()
    with open(marker,'a',encoding='utf8') as f:f.write(json.dumps({'uris':uris,'startup':startup,'mode':mode},ensure_ascii=False)+'\n')
    if mode=='fail':invocation.return_dbus_error('org.freedesktop.FileManager1.Refused','Isolated fixture refused reveal')
    elif mode=='ok':invocation.return_value(None)
    else:pending.append(invocation)
pending=[]
def acquired(connection,name):
    connection.register_object('/org/freedesktop/FileManager1',info,call,None,None)
    print('READY',flush=True)
owner=Gio.bus_own_name(Gio.BusType.SESSION,'org.freedesktop.FileManager1',Gio.BusNameOwnerFlags.NONE,None,acquired,None)
GLib.MainLoop().run()
