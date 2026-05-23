import Toybox.Lang;
import Toybox.System;
import Toybox.WatchUi;

public function dataView(
    boat as Boat
) as [ViewLoopFactory.Views, ViewLoopFactory.Delegates] {
    var menu = new WatchUi.Menu2({
        :dividerType => WatchUi.Menu2.DIVIDER_TYPE_ICON,
    });

    menu.addItem(menuItem("Depth", "0.0m", Rez.Drawables.DepthIcon));
    menu.addItem(menuItem("Water Temp", "0°F", Rez.Drawables.TemperatureIcon));

    return [menu, new DataViewDelegate(boat)];
}

public class DataViewDelegate extends WatchUi.Menu2InputDelegate {
    var boat;

    function initialize(boat as Boat) {
        Menu2InputDelegate.initialize();
        self.boat = boat;
    }

    function onSelect(item) {}

    function onPreviousPage() as Lang.Boolean {
        WatchUi.switchToView(
            new WindView(self.boat),
            new WindViewDelegate(self.boat),
            WatchUi.SLIDE_DOWN
        );
        return true;
    }

    function onNextPage() as Lang.Boolean {
        return true;
    }
}

function menuItem(name, value, icon) {
    return new WatchUi.IconMenuItem(
        name,
        value,
        null,
        new WatchUi.Bitmap({
            :rezId => icon,
            :locX => WatchUi.LAYOUT_HALIGN_CENTER,
            :locY => WatchUi.LAYOUT_VALIGN_CENTER,
        }),
        {}
    );
}
