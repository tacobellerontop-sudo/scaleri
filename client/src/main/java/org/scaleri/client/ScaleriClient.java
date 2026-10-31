package org.scaleri.client;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.fabricmc.fabric.api.client.message.v1.ClientReceiveMessageEvents;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.fabricmc.fabric.api.client.rendering.v1.WorldRenderEvents;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.fabricmc.fabric.api.client.screen.v1.Screens;
import net.fabricmc.fabric.api.event.player.AttackEntityCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.screen.GameMenuScreen;
import net.minecraft.client.gui.screen.TitleScreen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.text.Text;
import net.minecraft.util.ActionResult;
import org.lwjgl.glfw.GLFW;
import org.scaleri.client.config.ConfigManager;
import org.scaleri.client.gui.HudEditorScreen;
import org.scaleri.client.gui.ModMenuScreen;
import org.scaleri.client.gui.Theme;
import org.scaleri.client.module.ModuleManager;
import org.scaleri.client.util.CombatTracker;
import org.scaleri.client.util.SmokeTest;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class ScaleriClient implements ClientModInitializer {
    public static final String NAME = "Scaleri Client";
    public static final Logger LOGGER = LoggerFactory.getLogger(NAME);
    private static final String KEY_CATEGORY = "key.categories.scaleri";

    private static ScaleriClient instance;

    private ModuleManager modules;
    private ConfigManager config;
    private KeyBinding menuKey;
    private KeyBinding hudEditorKey;

    public static ScaleriClient get() {
        return instance;
    }

    public ModuleManager getModules() {
        return modules;
    }

    public ConfigManager getConfig() {
        return config;
    }

    public KeyBinding getMenuKey() {
        return menuKey;
    }

    @Override
    public void onInitializeClient() {
        instance = this;
        modules = new ModuleManager();
        config = new ConfigManager(modules);
        config.load();

        menuKey = register("key.scaleri.menu", GLFW.GLFW_KEY_RIGHT_SHIFT);
        hudEditorKey = register("key.scaleri.hud_editor", GLFW.GLFW_KEY_UNKNOWN);
        modules.getZoom().setKey(register("key.scaleri.zoom", GLFW.GLFW_KEY_C));
        modules.getFreelook().setKey(register("key.scaleri.freelook", GLFW.GLFW_KEY_LEFT_ALT));

        ClientTickEvents.START_CLIENT_TICK.register(client -> modules.getToggleSprint().onPreTick());
        ClientTickEvents.END_CLIENT_TICK.register(this::onEndTick);
        ClientLifecycleEvents.CLIENT_STOPPING.register(client -> config.save());

        HudRenderCallback.EVENT.register((context, tickCounter) ->
                modules.renderHud(context, tickCounter.getTickDelta(false)));

        WorldRenderEvents.BLOCK_OUTLINE.register((context, outline) -> {
            if (!modules.getBlockOutline().isEnabled()) {
                return true;
            }
            return modules.getBlockOutline().render(context, outline.vertexConsumer(), outline.blockPos(),
                    outline.blockState(), outline.cameraX(), outline.cameraY(), outline.cameraZ());
        });

        AttackEntityCallback.EVENT.register((player, world, hand, entity, hitResult) -> {
            if (world.isClient && player == MinecraftClient.getInstance().player) {
                CombatTracker.onAttack(entity);
                if (modules.getHitParticles().isEnabled()) {
                    modules.getHitParticles().onAttack(entity);
                }
            }
            return ActionResult.PASS;
        });

        ClientReceiveMessageEvents.GAME.register((message, overlay) -> {
            if (!overlay && modules.getAutoGg().isEnabled()) {
                modules.getAutoGg().onGameMessage(message.getString());
            }
        });

        ScreenEvents.AFTER_INIT.register((client, screen, width, height) -> {
            boolean title = screen instanceof TitleScreen;
            if ((title || screen instanceof GameMenuScreen) && modules.getTheme().menuButton.isOn()) {
                Screens.getButtons(screen).add(ButtonWidget.builder(Text.literal("Scaleri"),
                        b -> client.setScreen(new ModMenuScreen(screen))).dimensions(4, 4, 56, 20).build());
            }
            if (title) {
                ScreenEvents.afterRender(screen).register((s, ctx, mouseX, mouseY, delta) -> {
                    String mark = NAME + " " + version();
                    ctx.drawText(client.textRenderer, mark, s.width - client.textRenderer.getWidth(mark) - 4, 4, Theme.accent(), true);
                });
            }
        });

        if (SmokeTest.ENABLED) {
            SmokeTest.install();
        }
        LOGGER.info("{} {} loaded with {} modules", NAME, version(), modules.all().size());
    }

    private KeyBinding register(String id, int key) {
        return KeyBindingHelper.registerKeyBinding(new KeyBinding(id, InputUtil.Type.KEYSYM, key, KEY_CATEGORY));
    }

    private void onEndTick(MinecraftClient client) {
        CombatTracker.tick();
        modules.tick();
        while (menuKey.wasPressed()) {
            if (client.currentScreen == null) {
                client.setScreen(new ModMenuScreen(null));
            }
        }
        while (hudEditorKey.wasPressed()) {
            if (client.currentScreen == null) {
                client.setScreen(new HudEditorScreen(null));
            }
        }
    }

    public static String version() {
        return net.fabricmc.loader.api.FabricLoader.getInstance().getModContainer("scaleri")
                .map(c -> c.getMetadata().getVersion().getFriendlyString()).orElse("dev");
    }
}
