import Toybox.Application;
import Toybox.Lang;
import Toybox.WatchUi;
import Toybox.Timer;

class App extends Application.AppBase {
    var boat = new Boat();
    var timer = new Timer.Timer();

    function initialize() {
        AppBase.initialize();
    }

    function onStart(state as Dictionary?) as Void {
        timer.start(method(:onTimer), 1000, true);
        // self.boat.initialize();
    }

    function onStop(state as Dictionary?) as Void {}

    function getInitialView() as [Views] or [Views, InputDelegates] {
        return [new WindView(self.boat)];
    }

    function onTimer() {
        WatchUi.requestUpdate();
    }
}

function getApp() as App {
    return Application.getApp() as App;
}
