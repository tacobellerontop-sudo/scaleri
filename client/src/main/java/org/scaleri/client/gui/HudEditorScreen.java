package org.scaleri.client.gui;

import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.util.math.MathHelper;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.HudModule;

/** Drag HUD modules around, scroll to resize, right-click for settings. */
public class HudEditorScreen extends ScaleriScreen {
    private static final int SNAP = 4;

    private HudModule dragged;
    private double grabX;
    private double grabY;
    private boolean snappedX;
    private boolean snappedY;

    public HudEditorScreen(Screen parent) {
        super("HUD Editor", parent);
    }

    private List<HudModule> visible() {
        return ScaleriClient.get().getModules().hud().stream().filter(HudModule::isEnabled).toList();
    }

    @Override
    public void renderBackground(DrawContext context, int mouseX, int mouseY, float delta) {
        if (client != null && client.world == null) {
            super.renderBackground(context, mouseX, mouseY, delta);
        } else {
            context.fill(0, 0, width, height, 0x40000000);
        }
    }

    @Override
    protected void draw(DrawContext ctx, int mouseX, int mouseY, float delta) {
        if (dragged != null) {
            int guide = Theme.withAlpha(Theme.accent(), 0x90);
            if (snappedX) {
                ctx.fill(width / 2, 0, width / 2 + 1, height, guide);
            }
            if (snappedY) {
                ctx.fill(0, height / 2, width, height / 2 + 1, guide);
            }
        }

        HudModule hovered = hoveredModule(mouseX, mouseY);
        for (HudModule m : visible()) {
            m.renderPlaced(ctx, delta, true);
            int x = m.getScreenX(width);
            int y = m.getScreenY(height);
            boolean hot = m == hovered || m == dragged;
            Theme.outline(ctx, x - 1, y - 1, m.getScaledWidth() + 2, m.getScaledHeight() + 2,
                    hot ? Theme.accent() : 0x50FFFFFF);
            if (hot) {
                String label = m.getName() + "  " + Math.round(m.getScale() * 100) + "%";
                int ly = y > 12 ? y - 11 : y + m.getScaledHeight() + 3;
                ctx.drawText(textRenderer, label, MathHelper.clamp(x, 2, width - textRenderer.getWidth(label) - 2), ly, Theme.TEXT, true);
            }
        }

        // Bottom toolbar
        int bw = 80;
        int bx = width / 2 - bw - 4;
        int by = height - 54;
        Theme.roundRect(ctx, bx - 8, by - 6, bw * 2 + 24, 32, 6, Theme.withAlpha(Theme.PANEL, 0xE0));
        button(ctx, bx, by, bw, 20, "Modules", false, mouseX, mouseY, () -> client.setScreen(new ModMenuScreen(this)));
        button(ctx, bx + bw + 8, by, bw, 20, "Done", true, mouseX, mouseY, this::close);
        String hint = "Drag to move • Scroll to resize • Right-click for settings";
        ctx.drawText(textRenderer, hint, (width - textRenderer.getWidth(hint)) / 2, height - 16, Theme.TEXT_DIM, true);
    }

    private HudModule hoveredModule(double mx, double my) {
        List<HudModule> list = visible();
        for (int i = list.size() - 1; i >= 0; i--) {
            HudModule m = list.get(i);
            if (Theme.inside(mx, my, m.getScreenX(width), m.getScreenY(height), m.getScaledWidth(), m.getScaledHeight())) {
                return m;
            }
        }
        return null;
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (super.mouseClicked(mouseX, mouseY, button)) {
            return true;
        }
        HudModule m = hoveredModule(mouseX, mouseY);
        if (m == null) {
            return false;
        }
        if (button == 1) {
            client.setScreen(new ModSettingsScreen(this, m));
        } else if (button == 0) {
            dragged = m;
            grabX = mouseX - m.getScreenX(width);
            grabY = mouseY - m.getScreenY(height);
        }
        return true;
    }

    @Override
    public boolean mouseDragged(double mouseX, double mouseY, int button, double deltaX, double deltaY) {
        if (dragged == null) {
            return super.mouseDragged(mouseX, mouseY, button, deltaX, deltaY);
        }
        int w = dragged.getScaledWidth();
        int h = dragged.getScaledHeight();
        double x = MathHelper.clamp(mouseX - grabX, 0, width - w);
        double y = MathHelper.clamp(mouseY - grabY, 0, height - h);

        snappedX = Math.abs(x + w / 2.0 - width / 2.0) < SNAP;
        if (snappedX) {
            x = (width - w) / 2.0;
        } else if (x < SNAP) {
            x = 0;
        } else if (width - w - x < SNAP) {
            x = width - w;
        }
        snappedY = Math.abs(y + h / 2.0 - height / 2.0) < SNAP;
        if (snappedY) {
            y = (height - h) / 2.0;
        } else if (y < SNAP) {
            y = 0;
        } else if (height - h - y < SNAP) {
            y = height - h;
        }
        dragged.setPosition(x / width, y / height);
        return true;
    }

    @Override
    public boolean mouseReleased(double mouseX, double mouseY, int button) {
        dragged = null;
        snappedX = false;
        snappedY = false;
        return super.mouseReleased(mouseX, mouseY, button);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double horizontalAmount, double verticalAmount) {
        HudModule m = hoveredModule(mouseX, mouseY);
        if (m == null || verticalAmount == 0) {
            return false;
        }
        // Grow or shrink around the module's center.
        int cx = m.getScreenX(width) + m.getScaledWidth() / 2;
        int cy = m.getScreenY(height) + m.getScaledHeight() / 2;
        m.setScale(m.getScale() + (verticalAmount > 0 ? 0.05 : -0.05));
        m.setPosition((cx - m.getScaledWidth() / 2.0) / width, (cy - m.getScaledHeight() / 2.0) / height);
        return true;
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
