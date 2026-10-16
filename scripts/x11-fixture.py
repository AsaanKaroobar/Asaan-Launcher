# Scoped native test fixture; no external Python packages required.
import ctypes, sys, time
x=ctypes.CDLL('libX11.so.6');t=ctypes.CDLL('libXtst.so.6')
x.XOpenDisplay.restype=ctypes.c_void_p
x.XDefaultRootWindow.argtypes=[ctypes.c_void_p];x.XDefaultRootWindow.restype=ctypes.c_ulong
x.XStringToKeysym.argtypes=[ctypes.c_char_p];x.XStringToKeysym.restype=ctypes.c_ulong
x.XKeysymToKeycode.argtypes=[ctypes.c_void_p,ctypes.c_ulong];x.XKeysymToKeycode.restype=ctypes.c_uint
x.XFlush.argtypes=[ctypes.c_void_p]
x.XSync.argtypes=[ctypes.c_void_p,ctypes.c_int]
x.XGetInputFocus.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_ulong),ctypes.POINTER(ctypes.c_int)]
x.XSetInputFocus.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.c_int,ctypes.c_ulong]
x.XCreateSimpleWindow.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.c_int,ctypes.c_int,ctypes.c_uint,ctypes.c_uint,ctypes.c_uint,ctypes.c_ulong,ctypes.c_ulong];x.XCreateSimpleWindow.restype=ctypes.c_ulong
x.XStoreName.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.c_char_p]
x.XMapWindow.argtypes=[ctypes.c_void_p,ctypes.c_ulong]
t.XTestFakeKeyEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
d=x.XOpenDisplay(None)
if not d:raise RuntimeError('X11 display unavailable')
if sys.argv[1]=='fixture':
 w=x.XCreateSimpleWindow(d,x.XDefaultRootWindow(d),10,10,240,80,0,0,0x303535);x.XStoreName(d,w,b'Asaaan focus test fixture');x.XMapWindow(d,w);x.XFlush(d);time.sleep(.3);x.XSetInputFocus(d,w,1,0);x.XFlush(d);print(w,flush=True)
 while True:time.sleep(1)
elif sys.argv[1]=='ancestors':
 x.XQueryTree.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.POINTER(ctypes.c_ulong),ctypes.POINTER(ctypes.c_ulong),ctypes.POINTER(ctypes.POINTER(ctypes.c_ulong)),ctypes.POINTER(ctypes.c_uint)];x.XFree.argtypes=[ctypes.c_void_p]
 w=ctypes.c_ulong();revert=ctypes.c_int();x.XGetInputFocus(d,ctypes.byref(w),ctypes.byref(revert));ancestors=[]
 while w.value>1 and w.value not in ancestors:
  ancestors.append(w.value);root=ctypes.c_ulong();parent=ctypes.c_ulong();children=ctypes.POINTER(ctypes.c_ulong)();count=ctypes.c_uint()
  if not x.XQueryTree(d,w,ctypes.byref(root),ctypes.byref(parent),ctypes.byref(children),ctypes.byref(count)):break
  if children:x.XFree(children)
  w=parent
 print(ancestors)
elif sys.argv[1]=='state':
 x.XQueryKeymap.argtypes=[ctypes.c_void_p,ctypes.c_void_p];keys=ctypes.create_string_buffer(32);x.XQueryKeymap(d,keys);print([i for i in range(256) if keys.raw[i//8] & (1 << (i%8))])
elif sys.argv[1]=='focus':
 w=ctypes.c_ulong();revert=ctypes.c_int();x.XGetInputFocus(d,ctypes.byref(w),ctypes.byref(revert));print(w.value)
else:
 keys=[x.XKeysymToKeycode(d,x.XStringToKeysym(k.encode())) for k in sys.argv[2:]]
 for k in keys:t.XTestFakeKeyEvent(d,k,1,0)
 x.XFlush(d);time.sleep(.07)
 for k in reversed(keys):t.XTestFakeKeyEvent(d,k,0,0)
 x.XSync(d,0);time.sleep(.1)
