import Toybox.Application;
import Toybox.Lang;
import Toybox.WatchUi;
import Toybox.Math;

class App extends Application.AppBase {
    function initialize() {
        AppBase.initialize();
    }

    // onStart() is called on application start up
    function onStart(state as Dictionary?) as Void {}

    // onStop() is called when your application is exiting
    function onStop(state as Dictionary?) as Void {}

    // Return the initial view of your application here
    function getInitialView() as [Views] or [Views, InputDelegates] {
        return [new View()];
    }
}

const TAU = Math.PI * 2.0;
const PI_FRAC_2 = Math.PI / 2.0;
const PI_2_FRAC_3 = (Math.PI * 2.0) / 3.0;

class View extends WatchUi.View {
    var angle = 0.0;

    function initialize() {
        View.initialize();
    }

    function onUpdate(dc) {
        View.onUpdate(dc);

        var w = dc.getWidth();
        var h = dc.getHeight();

        var cx = w / 2;
        var cy = h / 2;

        dc.setColor(Graphics.COLOR_WHITE, Graphics.COLOR_BLACK);
        dc.setPenWidth(5);

        for (var i = 0; i < 12; i++) {
            var t = i / 12.0;
            var θ = t * TAU;
            var x = Math.cos(θ + PI_FRAC_2);
            var y = Math.sin(θ + PI_FRAC_2);

            dc.drawLine(
                cx + x * cx,
                cy + y * cy,
                cx + x * cx * 0.95,
                cy + y * cy * 0.95
            );
            // dc.fillCircle(cx + x * cx * 0.95, cy + y * cy * 0.95, 0.05);

            var degree = Math.round(Math.toDegrees(θ - Math.PI))
                .toNumber()
                .abs();
            if (degree <= 120) {
                dc.drawText(
                    cx + x * cx * 0.8,
                    cy + y * cy * 0.8,
                    Graphics.FONT_XTINY,
                    degree,
                    Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER
                );
            }
        }

        dc.drawText(
            cx,
            h * 0.75,
            Graphics.FONT_LARGE,
            "12.3",
            Graphics.TEXT_JUSTIFY_CENTER
        );

        var x = Math.cos(angle - PI_FRAC_2);
        var y = Math.sin(angle - PI_FRAC_2);
        var r = 6.0;
        var l = cx * 0.9;
        dc.fillPolygon([
            [cx + x * l, cy + y * l],
            [cy + (y * r), cx - (x * r)],
            [cy - (y * r), cx + (x * r)],
            
        ]);
        dc.fillCircle(cx, cy, r * 1.5);
    }
}

function getApp() as App {
    return Application.getApp() as App;
}
