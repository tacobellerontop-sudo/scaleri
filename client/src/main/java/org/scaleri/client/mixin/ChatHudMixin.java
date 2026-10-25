package org.scaleri.client.mixin;

import net.minecraft.client.gui.hud.ChatHud;
import net.minecraft.text.Text;
import org.scaleri.client.ScaleriClient;
import org.scaleri.client.module.misc.ChatTimestampsModule;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

@Mixin(ChatHud.class)
public class ChatHudMixin {
    @ModifyVariable(
            method = "addMessage(Lnet/minecraft/text/Text;Lnet/minecraft/network/message/MessageSignatureData;Lnet/minecraft/client/gui/hud/MessageIndicator;)V",
            at = @At("HEAD"), argsOnly = true)
    private Text scaleri$timestamp(Text message) {
        ScaleriClient client = ScaleriClient.get();
        if (client != null) {
            ChatTimestampsModule stamps = client.getModules().getChatTimestamps();
            if (stamps.isEnabled()) {
                return stamps.decorate(message);
            }
        }
        return message;
    }
}
