package org.scaleri.client.module;

import net.minecraft.client.gui.DrawContext;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.setting.BoolSetting;
import org.scaleri.client.setting.ColorSetting;
import org.scaleri.client.setting.ModeSetting;

/** A HUD module that shows one line of text, like "FPS: 240". */
public abstract class TextHudModule extends HudModule {
    protected final BoolSetting background = add(new BoolSetting("Background", true));
    protected final BoolSetting shadow = add(new BoolSetting("Text Shadow", true));
    protected final ModeSetting brackets = add(new ModeSetting("Brackets", "None", "None", "[ ]", "( )", "< >"));
    protected final ColorSetting textColor = add(new ColorSetting("Text Color", 0xFFFFFFFF));
    protected final ColorSetting backgroundColor = add(new ColorSetting("Background Color", 0x6F000000));

    protected TextHudModule(String name, String description, boolean enabledByDefault, double x, double y) {
        super(name, description, enabledByDefault, x, y);
    }

    protected TextHudModule(String name, String description, Category category, boolean enabledByDefault, double x, double y) {
        super(name, description, category, enabledByDefault, x, y);
    }

    /** The text to show, or null to hide the module this frame. */
    protected abstract String getText(boolean editing);

    @Override
    public void render(DrawContext context, float tickDelta, boolean editing) {
        String text = getText(editing);
        if (text == null) {
            if (editing) {
                text = getName();
            } else {
                return;
            }
        }
        text = switch (brackets.get()) {
            case "[ ]" -> "[" + text + "]";
            case "( )" -> "(" + text + ")";
            case "< >" -> "<" + text + ">";
            default -> text;
        };
        int textWidth = mc.textRenderer.getWidth(text);
        if (background.isOn()) {
            width = Math.max(textWidth + 10, 56);
            height = 16;
            Theme.roundRect(context, 0, 0, width, height, 3, backgroundColor.getArgb());
            context.drawText(mc.textRenderer, text, (width - textWidth) / 2, 4, textColor.getArgb(), shadow.isOn());
        } else {
            width = textWidth;
            height = 9;
            context.drawText(mc.textRenderer, text, 0, 0, textColor.getArgb(), shadow.isOn());
        }
    }
}
