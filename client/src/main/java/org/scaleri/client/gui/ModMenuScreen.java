package org.scaleri.client.gui;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.text.OrderedText;
import net.minecraft.text.Text;
import net.minecraft.util.math.MathHelper;
import org.lwjgl.glfw.GLFW;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.Category;
import org.scaleri.client.module.Module;

/** The main Scaleri menu: browse, search and toggle modules. */
public class ModMenuScreen extends ScaleriScreen {
    private static final int SIDEBAR = 96;
    private static final int CARD_MIN_WIDTH = 128;
    private static final int CARD_HEIGHT = 50;
    private static final int GAP = 6;

    private static Category lastCategory;

    private Category category = lastCategory;
    private String search = "";
    private double scroll;
    private double maxScroll;
    private final Map<Module, Float> anim = new HashMap<>();
    private long lastFrame = System.nanoTime();

    public ModMenuScreen(Screen parent) {
        super("Scaleri", parent);
    }

    @Override
    protected void draw(DrawContext ctx, int mouseX, int mouseY, float delta) {
        long now = System.nanoTime();
        float step = Math.min(1f, (now - lastFrame) / 1_000_000_000f * 14f);
        lastFrame = now;

        int pw = Math.min(width - 24, 520);
        int ph = Math.min(height - 24, 320);
        int px = (width - pw) / 2;
        int py = (height - ph) / 2;

        Theme.roundRect(ctx, px, py, pw, ph, 6, Theme.PANEL);
        Theme.roundRect(ctx, px, py, SIDEBAR, ph, 6, Theme.SIDEBAR);
        ctx.fill(px + SIDEBAR - 6, py, px + SIDEBAR, py + ph, Theme.SIDEBAR);

        // Sidebar
        int sx = px + 10;
        ctx.drawText(textRenderer, "Scaleri", sx, py + 12, Theme.accent(), true);
        ctx.drawText(textRenderer, "Client " + ScaleriClient.version(), sx, py + 23, Theme.TEXT_FAINT, false);

        int cy = py + 44;
        cy = categoryTab(ctx, sx - 4, cy, null, "All Modules", mouseX, mouseY);
        for (Category c : Category.values()) {
            cy = categoryTab(ctx, sx - 4, cy, c, c.getDisplayName(), mouseX, mouseY);
        }

        int bw = SIDEBAR - 12;
        button(ctx, px + 6, py + ph - 52, bw, 20, "Edit HUD", true, mouseX, mouseY,
                () -> client.setScreen(new HudEditorScreen(this)));
        button(ctx, px + 6, py + ph - 28, bw, 20, "Close", false, mouseX, mouseY, this::close);

        // Header
        int cx = px + SIDEBAR + 12;
        int cw = pw - SIDEBAR - 24;
        ctx.drawText(textRenderer, category == null ? "All Modules" : category.getDisplayName(), cx, py + 14, Theme.TEXT, true);

        int fieldW = Math.min(140, cw / 2);
        int fx = cx + cw - fieldW;
        Theme.roundRect(ctx, fx, py + 9, fieldW, 18, 4, Theme.FIELD);
        boolean blink = (System.currentTimeMillis() / 500) % 2 == 0;
        String shown = search.isEmpty() ? "Search..." : textRenderer.trimToWidth(search, fieldW - 14);
        ctx.drawText(textRenderer, shown + (blink && !search.isEmpty() ? "_" : ""), fx + 6, py + 14,
                search.isEmpty() ? Theme.TEXT_FAINT : Theme.TEXT, false);

        // Module grid
        List<Module> shownModules = filtered();
        int top = py + 36;
        int bottom = py + ph - 8;
        int cols = Math.max(1, (cw + GAP) / (CARD_MIN_WIDTH + GAP));
        int cardW = (cw - GAP * (cols - 1)) / cols;
        int rows = (shownModules.size() + cols - 1) / cols;
        maxScroll = Math.max(0, rows * (CARD_HEIGHT + GAP) - GAP - (bottom - top));
        scroll = MathHelper.clamp(scroll, 0, maxScroll);

        ctx.enableScissor(cx - 2, top, cx + cw + 2, bottom);
        clip(top, bottom);
        for (int i = 0; i < shownModules.size(); i++) {
            Module m = shownModules.get(i);
            int x = cx + (i % cols) * (cardW + GAP);
            int y = top + (i / cols) * (CARD_HEIGHT + GAP) - (int) scroll;
            if (y + CARD_HEIGHT < top || y > bottom) {
                continue;
            }
            card(ctx, m, x, y, cardW, mouseX, mouseY, step);
        }
        if (shownModules.isEmpty()) {
            String msg = "No modules match \"" + search + "\"";
            ctx.drawText(textRenderer, msg, cx + (cw - textRenderer.getWidth(msg)) / 2, top + 30, Theme.TEXT_DIM, false);
        }
        unclip();
        ctx.disableScissor();

        if (maxScroll > 0) {
            int trackH = bottom - top;
            int thumbH = Math.max(20, (int) (trackH * (trackH / (trackH + maxScroll))));
            int thumbY = top + (int) ((trackH - thumbH) * (scroll / maxScroll));
            ctx.fill(px + pw - 5, thumbY, px + pw - 3, thumbY + thumbH, Theme.withAlpha(Theme.TEXT_DIM, 0x80));
        }
    }

    private int categoryTab(DrawContext ctx, int x, int y, Category c, String label, int mouseX, int mouseY) {
        int w = SIDEBAR - 12;
        boolean selected = category == c;
        boolean hover = Theme.inside(mouseX, mouseY, x, y, w, 18);
        if (selected) {
            Theme.roundRect(ctx, x, y, w, 18, 4, Theme.withAlpha(Theme.accent(), 0x40));
            ctx.fill(x, y + 4, x + 2, y + 14, Theme.accent());
        } else if (hover) {
            Theme.roundRect(ctx, x, y, w, 18, 4, 0x18FFFFFF);
        }
        ctx.drawText(textRenderer, label, x + 8, y + 5, selected ? Theme.TEXT : Theme.TEXT_DIM, false);
        onClick(x, y, w, 18, (mx, my, b) -> {
            category = c;
            lastCategory = c;
            scroll = 0;
        });
        return y + 21;
    }

    private void card(DrawContext ctx, Module m, int x, int y, int w, int mouseX, int mouseY, float step) {
        boolean hover = Theme.inside(mouseX, mouseY, x, y, w, CARD_HEIGHT) && mouseY >= y;
        Theme.roundRect(ctx, x, y, w, CARD_HEIGHT, 5, hover ? Theme.CARD_HOVER : Theme.CARD);

        float target = m.isEnabled() ? 1f : 0f;
        float a = anim.getOrDefault(m, target);
        a += (target - a) * step;
        anim.put(m, a);

        if (m.isToggleable()) {
            ctx.fill(x, y + 8, x + 2, y + CARD_HEIGHT - 8, Theme.blend(Theme.OFF, Theme.accent(), a));
        }
        ctx.drawText(textRenderer, m.getName(), x + 9, y + 8, Theme.TEXT, false);
        List<OrderedText> lines = textRenderer.wrapLines(Text.literal(m.getDescription()), w - 18);
        for (int i = 0; i < Math.min(2, lines.size()); i++) {
            ctx.drawText(textRenderer, lines.get(i), x + 9, y + 20 + i * 10, Theme.TEXT_DIM, false);
        }

        boolean hasSettings = !m.getSettings().isEmpty();
        int toggleX = x + w - 28;
        int toggleY = y + 7;
        if (m.isToggleable()) {
            Theme.toggle(ctx, toggleX, toggleY, a);
        }
        // Click the card to toggle; click the gear (or a settings-only module) to open its settings.
        onClick(x, y, w, CARD_HEIGHT, (mx, my, b) -> {
            if (b == 1 || !m.isToggleable()) {
                if (hasSettings) {
                    client.setScreen(new ModSettingsScreen(this, m));
                }
            } else {
                m.toggle();
            }
        });
        if (hasSettings) {
            int gx = m.isToggleable() ? toggleX - 16 : x + w - 18;
            boolean gearHover = Theme.inside(mouseX, mouseY, gx - 2, toggleY - 2, 14, 14);
            ctx.drawText(textRenderer, "⚙", gx + 1, toggleY + 1, gearHover ? Theme.accent() : Theme.TEXT_FAINT, false);
            onClick(gx - 2, toggleY - 2, 14, 14, (mx, my, b) -> client.setScreen(new ModSettingsScreen(this, m)));
        }
    }

    private List<Module> filtered() {
        String q = search.toLowerCase(Locale.ROOT).trim();
        List<Module> list = new ArrayList<>();
        for (Module m : ScaleriClient.get().getModules().all()) {
            if (category != null && m.getCategory() != category) {
                continue;
            }
            if (!q.isEmpty() && !m.getName().toLowerCase(Locale.ROOT).contains(q)
                    && !m.getDescription().toLowerCase(Locale.ROOT).contains(q)) {
                continue;
            }
            list.add(m);
        }
        return list;
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double horizontalAmount, double verticalAmount) {
        scroll = MathHelper.clamp(scroll - verticalAmount * 24, 0, maxScroll);
        return true;
    }

    @Override
    public boolean charTyped(char chr, int modifiers) {
        if (chr >= ' ' && search.length() < 32) {
            search += chr;
            scroll = 0;
            return true;
        }
        return super.charTyped(chr, modifiers);
    }

    @Override
    public boolean keyPressed(int keyCode, int scanCode, int modifiers) {
        if (keyCode == GLFW.GLFW_KEY_BACKSPACE && !search.isEmpty()) {
            search = (modifiers & GLFW.GLFW_MOD_CONTROL) != 0 ? "" : search.substring(0, search.length() - 1);
            return true;
        }
        if (ScaleriClient.get().getMenuKey().matchesKey(keyCode, scanCode)) {
            close();
            return true;
        }
        return super.keyPressed(keyCode, scanCode, modifiers);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}
