import Toybox.Application;
import Toybox.Lang;
import Toybox.WatchUi;
import Toybox.Math;
import Toybox.Graphics;

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

const RED = Graphics.createColor(255, 0xe7, 0x25, 0x2e);
const GREEN = Graphics.createColor(255, 0x06, 0xa8, 0x4f);
const DARK_GRAY = Graphics.createColor(255, 0x32, 0x32, 0x32);

class View extends WatchUi.View {
    var angle = 0.0;
    var speed = 0.0;

    function initialize() {
        View.initialize();
    }

    function onUpdate(dc) {
        View.onUpdate(dc);

        var w = dc.getWidth();
        var h = dc.getHeight();

        var cx = w / 2;
        var cy = h / 2;

        dc.setPenWidth(cx * 0.1);
        dc.setColor(GREEN, Graphics.COLOR_BLACK);
        dc.drawArc(cx, cy, cx, Graphics.ARC_COUNTER_CLOCKWISE, 30, 70);
        dc.setColor(RED, Graphics.COLOR_BLACK);
        dc.drawArc(cx, cy, cx, Graphics.ARC_COUNTER_CLOCKWISE, 110, 150);

        dc.setPenWidth(5);
        for (var i = 0; i < 24; i++) {
            var t = i / 24.0;
            var θ = t * TAU;
            var x = Math.cos(θ + PI_FRAC_2);
            var y = Math.sin(θ + PI_FRAC_2);

            if (i % 2 == 0) {
                dc.setColor(Graphics.COLOR_WHITE, Graphics.COLOR_BLACK);
                var degree = Math.round(Math.toDegrees(θ - Math.PI))
                    .toNumber()
                    .abs();
                if (degree <= 120) {
                    dc.drawText(
                        cx + x * cx * 0.8,
                        cy + y * cy * 0.8,
                        Graphics.FONT_XTINY,
                        degree,
                        Graphics.TEXT_JUSTIFY_CENTER |
                            Graphics.TEXT_JUSTIFY_VCENTER
                    );
                }

                dc.drawLine(
                    cx + x * cx,
                    cy + y * cy,
                    cx + x * cx * 0.95,
                    cy + y * cy * 0.95
                );
            } else {
                dc.setColor(Graphics.COLOR_LT_GRAY, Graphics.COLOR_BLACK);
                dc.drawLine(
                    cx + x * cx,
                    cy + y * cy,
                    cx + x * cx * 0.98,
                    cy + y * cy * 0.98
                );
            }
        }

        dc.setColor(Graphics.COLOR_WHITE, Graphics.COLOR_BLACK);
        dc.drawText(
            cx,
            h * 0.7,
            font(100),
            speed.format("%.1f"),
            Graphics.TEXT_JUSTIFY_CENTER
        );
        dc.drawText(
            cx,
            h * 0.7 + 95,
            font(20),
            "KTS",
            Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER
        );

        var x = Math.cos(angle - PI_FRAC_2);
        var y = Math.sin(angle - PI_FRAC_2);
        var r = 6.0;
        var l = cx * 0.9;
        dc.fillPolygon([
            [cx + x * l, cy + y * l],
            [cy + y * r, cx - x * r],
            [cy - y * r, cx + x * r],
        ]);
        dc.fillCircle(cx, cy, r * 2);
        dc.setColor(DARK_GRAY, Graphics.COLOR_BLACK);
        dc.drawCircle(cx, cy, r * 2);
    }
}

function font(size as Number) as Graphics.VectorFont {
    return Graphics.getVectorFont({
        :face => ["RobotoCondensedBold", "RobotoRegular"],
        :size => size,
    });
}

// drawArc(x as Lang.Numeric, y as Lang.Numeric, r as Lang.Numeric, attr as Graphics.ArcDirection, degreeStart as Lang.Numeric, degreeEnd as Lang.Numeric) as Void

function getApp() as App {
    return Application.getApp() as App;
}
