ObjC.import('AppKit');
function run(argv){
    const app=$.NSApplication.sharedApplication;
    app.setActivationPolicy($.NSApplicationActivationPolicyAccessory);
    app.activateIgnoringOtherApps(true);
    const alert=$.NSAlert.alloc.init;
    alert.messageText='Force “'+argv[0]+'” to quit?';
    alert.informativeText="You will lose any changes you haven't saved.";
    alert.alertStyle=$.NSAlertStyleWarning;
    alert.addButtonWithTitle('Cancel');
    alert.addButtonWithTitle('Force Quit');
    return alert.runModal===$.NSAlertSecondButtonReturn?'yes':'no';
}
