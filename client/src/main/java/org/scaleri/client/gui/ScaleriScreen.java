package org.scaleri.client.gui;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.Text;
import org.scaleri.client.ScaleriClient;

/**
 * Base for Scaleri screens. Instead of persistent widgets, each frame registers
 * clickable regions while drawing, which keeps scrolling and resizing simple.
 */
public abstract class ScaleriScreen extends Screen {
    protected final Screen parent;
    private final List<Region> regions = new ArrayList<>();
    private Region dragging;

    protected ScaleriScreen(String title, Screen parent) {
        super(Text.literal(title));
        this.parent = parent;
    }

    public interface ClickAction {
        void run(double mouseX, double mouseY, int button);
    }

    public interface DragAction {
        void run(double mouseX, double mouseY);
    }

    private record Region(int x, int y, int w, int h, int clipTop, int clipBottom, ClickAction click, DragAction drag) {
        boolean contains(double mx, double my) {
            return Theme.inside(mx, my, x, y, w, h) && my >= clipTop && my < clipBottom;
        }
    }

    private int clipTop = Integer.MIN_VALUE;
    private int clipBottom = Integer.MAX_VALUE;

    /** Regions added after this only react inside the given vertical band (for scroll areas). */
    protected void clip(int top, int bottom) {
        clipTop = top;
        clipBottom = bottom;
    }

    protected void unclip() {
        clipTop = Integer.MIN_VALUE;
        clipBottom = Integer.MAX_VALUE;
    }

    protected void onClick(int x, int y, int w, int h, ClickAction action) {
        regions.add(new Region(x, y, w, h, clipTop, clipBottom, action, null));
    }

    /** A region that keeps receiving the mouse position while the button is held, like a slider. */
    protected void onDrag(int x, int y, int w, int h, DragAction action) {
        regions.add(new Region(x, y, w, h, clipTop, clipBottom, (mx, my, b) -> action.run(mx, my), action));
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        regions.clear();
        unclip();
        super.render(context, mouseX, mouseY, delta);
        draw(context, mouseX, mouseY, delta);
        unclip();
    }

    protected abstract void draw(DrawContext context, int mouseX, int mouseY, float delta);

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (super.mouseClicked(mouseX, mouseY, button)) {
            return true;
        }
        // Later regions are drawn on top, so check them first.
        for (int i = regions.size() - 1; i >= 0; i--) {
            Region r = regions.get(i);
            if (r.contains(mouseX, mouseY)) {
                if (r.drag != null) {
                    if (button != 0) {
                        continue;
                    }
                    dragging = r;
                }
                r.click.run(mouseX, mouseY, button);
                return true;
            }
        }
        return false;
    }

    @Override
    public boolean mouseDragged(double mouseX, double mouseY, int button, double deltaX, double deltaY) {
        if (dragging != null) {
            dragging.drag.run(mouseX, mouseY);
            return true;
        }
        return super.mouseDragged(mouseX, mouseY, button, deltaX, deltaY);
    }

    @Override
    public boolean mouseReleased(double mouseX, double mouseY, int button) {
        dragging = null;
        return super.mouseReleased(mouseX, mouseY, button);
    }

    @Override
    public void close() {
        ScaleriClient.get().getConfig().save();
        if (client != null) {
            client.setScreen(parent);
        }
    }

    /** A rounded button with centered text. */
    protected void button(DrawContext ctx, int x, int y, int w, int h, String label, boolean primary,
                          int mouseX, int mouseY, Runnable action) {
        boolean hover = Theme.inside(mouseX, mouseY, x, y, w, h);
        int bg = primary ? Theme.accent() : Theme.CARD;
        if (hover) {
            bg = Theme.blend(bg, 0xFFFFFFFF, 0.12f);
        }
        Theme.roundRect(ctx, x, y, w, h, 4, bg);
        int tw = textRenderer.getWidth(label);
        ctx.drawText(textRenderer, label, x + (w - tw) / 2, y + (h - 8) / 2, Theme.TEXT, false);
        onClick(x, y, w, h, (mx, my, b) -> action.run());
    }
}
