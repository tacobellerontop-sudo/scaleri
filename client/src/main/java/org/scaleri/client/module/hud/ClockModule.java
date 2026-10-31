package org.scaleri.client.module.hud;

import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import org.scaleri.client.module.TextHudModule;
import org.scaleri.client.setting.ModeSetting;

public class ClockModule extends TextHudModule {
    private static final DateTimeFormatter TWELVE = DateTimeFormatter.ofPattern("h:mm a");
    private static final DateTimeFormatter TWENTY_FOUR = DateTimeFormatter.ofPattern("HH:mm");

    private final ModeSetting format = add(new ModeSetting("Format", "12 hour", "12 hour", "24 hour"));

    public ClockModule() {
        super("Clock", "Shows the real-world time.", false, 0.004, 0.290);
    }

    @Override
    protected String getText(boolean editing) {
        return LocalTime.now().format(format.is("12 hour") ? TWELVE : TWENTY_FOUR);
    }
}
