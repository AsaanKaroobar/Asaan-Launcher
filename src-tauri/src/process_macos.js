// Floe-compatible native bundle URL identity. No process-name matching or shell.
ObjC.import('AppKit');
function run(argv) {
    const path=argv[0], mode=argv[1];
    const apps=$.NSWorkspace.sharedWorkspace.runningApplications.js.filter(app=>
        !app.isTerminated && app.bundleURL &&
        app.bundleURL.URLByResolvingSymlinksInPath.path.js===path);
    if(apps.length===0)return 'stopped';
    if(apps.length!==1)throw Error('Ambiguous application identity');
    const app=apps[0];
    const launched=app.launchDate;
    const stamp=launched?Number(launched.timeIntervalSince1970):NaN;
    if(!Number.isFinite(stamp)||stamp<=0)throw Error('Application launch identity unavailable');
    const token='mac:'+app.processIdentifier+':'+stamp;
    if(mode!=='inspect' && argv[2]!==token)throw Error('Application instance changed');
    if(mode==='hide' && !app.hide)throw Error('Hide refused');
    if(mode==='quit' && !app.terminate)throw Error('Quit refused');
    return JSON.stringify({hidden:!!app.isHidden,canQuit:true,token:token});
}
