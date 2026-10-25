package org.scaleri.client.module.visual;

import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;

public class ClearWeatherModule extends Module {
    public ClearWeatherModule() {
        super("Clear Weather", "Hides rain, snow and thunderstorms. Only affects you.", Category.VISUAL, false);
    }
}
