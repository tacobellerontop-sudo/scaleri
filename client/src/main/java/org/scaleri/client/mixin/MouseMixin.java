package org.scaleri.client.mixin;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.Mouse;
import net.minecraft.client.network.ClientPlayerEntity;
import org.lwjgl.glfw.GLFW;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.visual.FreelookModule;
import org.scaleri.client.util.ClickTracker;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Mouse.class)
public class MouseMixin {
    @Inject(method = "onMouseButton", at = @At("HEAD"))
    private void scaleri$countClick(long window, int button, int action, int mods, CallbackInfo ci) {
        MinecraftClient mc = MinecraftClient.getInstance();
        if (action == GLFW.GLFW_PRESS && window == mc.getWindow().getHandle() && mc.currentScreen == null) {
            ClickTracker.onClick(button);
        }
    }

    @Inject(method = "onMouseScroll", at = @At("HEAD"), cancellable = true)
    private void scaleri$zoomScroll(long window, double horizontal, double vertical, CallbackInfo ci) {
        ScaleriClient client = ScaleriClient.get();
        if (client != null && MinecraftClient.getInstance().currentScreen == null
                && client.getModules().getZoom().onScroll(vertical)) {
            ci.cancel();
        }
    }

    @Redirect(method = "updateMouse", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/client/network/ClientPlayerEntity;changeLookDirection(DD)V"))
    private void scaleri$freelook(ClientPlayerEntity player, double dx, double dy) {
        FreelookModule freelook = ScaleriClient.get().getModules().getFreelook();
        if (freelook.isEnabled() && freelook.isActive()) {
            freelook.turn(dx, dy);
        } else {
            player.changeLookDirection(dx, dy);
        }
    }
}
