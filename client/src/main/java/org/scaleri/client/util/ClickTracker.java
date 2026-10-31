package org.scaleri.client.util;

import java.util.ArrayDeque;
import java.util.Deque;
import org.lwjgl.glfw.GLFW;

/** Remembers recent mouse clicks so the HUD can show clicks per second. */
public final class ClickTracker {
    private static final Deque<Long> LEFT = new ArrayDeque<>();
    private static final Deque<Long> RIGHT = new ArrayDeque<>();

    private ClickTracker() {
    }

    public static synchronized void onClick(int button) {
        if (button == GLFW.GLFW_MOUSE_BUTTON_LEFT) {
            LEFT.addLast(System.currentTimeMillis());
        } else if (button == GLFW.GLFW_MOUSE_BUTTON_RIGHT) {
            RIGHT.addLast(System.currentTimeMillis());
        }
    }

    public static synchronized int getLeftCps() {
        return count(LEFT);
    }

    public static synchronized int getRightCps() {
        return count(RIGHT);
    }

    private static int count(Deque<Long> clicks) {
        long cutoff = System.currentTimeMillis() - 1000L;
        while (!clicks.isEmpty() && clicks.peekFirst() < cutoff) {
            clicks.removeFirst();
        }
        return clicks.size();
    }
}
