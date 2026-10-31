package org.scaleri.client.module;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import org.scaleri.client.module.hud.*;
import org.scaleri.client.module.misc.*;
import org.scaleri.client.module.pvp.*;
import org.scaleri.client.module.visual.*;

public class ModuleManager {
    private final List<Module> modules = new ArrayList<>();

    private final ThemeModule theme = add(new ThemeModule());
    private final ToggleSprintModule toggleSprint;
    private final CrosshairModule crosshair;
    private final NoHurtCamModule noHurtCam;
    private final LowFireModule lowFire;
    private final BlockOutlineModule blockOutline;
    private final HitParticlesModule hitParticles;
    private final ZoomModule zoom;
    private final FreelookModule freelook;
    private final FullbrightModule fullbright;
    private final TimeChangerModule timeChanger;
    private final ClearWeatherModule clearWeather;
    private final PotionEffectsModule potionEffects;
    private final ChatTimestampsModule chatTimestamps;
    private final AutoGgModule autoGg;

    public ModuleManager() {
        add(new FpsModule());
        add(new CpsModule());
        add(new PingModule());
        add(new CoordinatesModule());
        add(new KeystrokesModule());
        add(new ArmorStatusModule());
        potionEffects = add(new PotionEffectsModule());
        add(new ReachModule());
        add(new ComboModule());
        add(new ClockModule());
        add(new SpeedModule());
        add(new MemoryModule());
        add(new ServerAddressModule());
        add(new SessionTimerModule());
        add(new DirectionModule());
        add(new ItemCounterModule());
        add(new CooldownsModule());

        toggleSprint = add(new ToggleSprintModule());
        crosshair = add(new CrosshairModule());
        noHurtCam = add(new NoHurtCamModule());
        lowFire = add(new LowFireModule());
        blockOutline = add(new BlockOutlineModule());
        hitParticles = add(new HitParticlesModule());

        zoom = add(new ZoomModule());
        freelook = add(new FreelookModule());
        fullbright = add(new FullbrightModule());
        timeChanger = add(new TimeChangerModule());
        clearWeather = add(new ClearWeatherModule());

        chatTimestamps = add(new ChatTimestampsModule());
        autoGg = add(new AutoGgModule());
        add(new DeathCoordsModule());
    }

    private <M extends Module> M add(M module) {
        modules.add(module);
        return module;
    }

    public List<Module> all() {
        return Collections.unmodifiableList(modules);
    }

    public List<HudModule> hud() {
        List<HudModule> list = new ArrayList<>();
        for (Module m : modules) {
            if (m instanceof HudModule h) {
                list.add(h);
            }
        }
        return list;
    }

    public void tick() {
        for (Module m : modules) {
            if (m.isEnabled()) {
                m.onTick();
            }
        }
    }

    public void renderHud(DrawContext context, float tickDelta) {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (mc.options.hudHidden || mc.getDebugHud().shouldShowDebugHud() || mc.currentScreen instanceof org.scaleri.client.gui.HudEditorScreen) {
            return;
        }
        for (HudModule m : hud()) {
            if (m.isEnabled()) {
                m.renderPlaced(context, tickDelta, false);
            }
        }
    }

    /** Only true if the module exists and is switched on. */
    public static boolean on(Module m) {
        return m != null && m.isEnabled();
    }

    public ThemeModule getTheme() {
        return theme;
    }

    public ToggleSprintModule getToggleSprint() {
        return toggleSprint;
    }

    public CrosshairModule getCrosshair() {
        return crosshair;
    }

    public NoHurtCamModule getNoHurtCam() {
        return noHurtCam;
    }

    public LowFireModule getLowFire() {
        return lowFire;
    }

    public BlockOutlineModule getBlockOutline() {
        return blockOutline;
    }

    public HitParticlesModule getHitParticles() {
        return hitParticles;
    }

    public ZoomModule getZoom() {
        return zoom;
    }

    public FreelookModule getFreelook() {
        return freelook;
    }

    public FullbrightModule getFullbright() {
        return fullbright;
    }

    public TimeChangerModule getTimeChanger() {
        return timeChanger;
    }

    public ClearWeatherModule getClearWeather() {
        return clearWeather;
    }

    public PotionEffectsModule getPotionEffects() {
        return potionEffects;
    }

    public ChatTimestampsModule getChatTimestamps() {
        return chatTimestamps;
    }

    public AutoGgModule getAutoGg() {
        return autoGg;
    }
}
