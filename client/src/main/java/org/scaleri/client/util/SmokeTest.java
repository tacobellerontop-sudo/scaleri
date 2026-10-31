package org.scaleri.client.util;

import java.util.ArrayDeque;
import java.util.Deque;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.Screen;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.gui.HudEditorScreen;
import org.scaleri.client.gui.ModMenuScreen;
import org.scaleri.client.gui.ModSettingsScreen;
import org.scaleri.client.module.Module;
import org.spongepowered.asm.mixin.MixinEnvironment;

/**
 * CI check, enabled with {@code -Dscaleri.smokeTest=true}: once the title screen is up,
 * force every mixin to apply, open every Scaleri screen for a few frames, then quit.
 * Any failure crashes the game, which fails the CI job.
 */
public final class SmokeTest {
    public static final boolean ENABLED = Boolean.getBoolean("scaleri.smokeTest");
    private static final int TICKS_PER_SCREEN = 5;

    private static final Deque<Screen> queue = new ArrayDeque<>();
    private static boolean started;
    private static int wait;

    private SmokeTest() {
    }

    public static void install() {
        ClientTickEvents.END_CLIENT_TICK.register(SmokeTest::tick);
    }

    private static void tick(MinecraftClient mc) {
        if (!started) {
            // First launch shows the accessibility screen instead of the title screen,
            // so start from whatever screen is up once loading has finished.
            if (mc.getOverlay() != null || mc.currentScreen == null || ++wait < 40) {
                return;
            }
            started = true;
            ScaleriClient.LOGGER.info("[smoke] auditing mixins");
            MixinEnvironment.getCurrentEnvironment().audit();

            // Turn every module on so the editor draws them all.
            for (Module m : ScaleriClient.get().getModules().all()) {
                m.setEnabled(true);
            }
            Screen title = mc.currentScreen;
            ModMenuScreen menu = new ModMenuScreen(title);
            queue.add(menu);
            for (Module m : ScaleriClient.get().getModules().all()) {
                queue.add(new ModSettingsScreen(menu, m));
            }
            queue.add(new HudEditorScreen(menu));
            queue.add(title);
            wait = 0;
            return;
        }
        if (++wait < TICKS_PER_SCREEN) {
            return;
        }
        wait = 0;
        Screen next = queue.poll();
        if (next != null) {
            ScaleriClient.LOGGER.info("[smoke] showing {}", next.getTitle().getString());
            mc.setScreen(next);
            return;
        }
        ScaleriClient.LOGGER.info("SCALERI_SMOKE_OK");
        mc.scheduleStop();
    }
}
