package org.scaleri.client.module.misc;

import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import net.minecraft.text.MutableText;
import net.minecraft.text.Text;
import net.minecraft.text.TextColor;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;
import org.scaleri.client.setting.ColorSetting;
import org.scaleri.client.setting.ModeSetting;

public class ChatTimestampsModule extends Module {
    private final ModeSetting format = add(new ModeSetting("Format", "12 hour", "12 hour", "24 hour", "24 hour + seconds"));
    private final ColorSetting color = add(new ColorSetting("Color", 0xFF8A8F99));

    public ChatTimestampsModule() {
        super("Chat Timestamps", "Adds the time to the start of each chat message.", Category.MISC, false);
    }

    public Text decorate(Text message) {
        String pattern = switch (format.get()) {
            case "24 hour" -> "HH:mm";
            case "24 hour + seconds" -> "HH:mm:ss";
            default -> "h:mm";
        };
        MutableText stamp = Text.literal("[" + LocalTime.now().format(DateTimeFormatter.ofPattern(pattern)) + "] ")
                .styled(s -> s.withColor(TextColor.fromRgb(color.get() & 0xFFFFFF)));
        return Text.empty().append(stamp).append(message);
    }
}
