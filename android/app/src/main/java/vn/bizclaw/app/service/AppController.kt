package vn.bizclaw.app.service

import android.content.Context
import android.content.Intent
import android.net.Uri
import kotlinx.coroutines.delay

/**
 * AppController — high-level automation for popular apps.
 *
 * Uses BizClawAccessibilityService to control apps like Facebook, Messenger, Zalo.
 * Each method is a complete "workflow" that agents can call as a single tool.
 *
 * ⚙️ Architecture:
 *   Agent tool call "facebook.post"
 *       → AppController.facebookPost()
 *           → Open Facebook app
 *           → Find "Bạn đang nghĩ gì?" field
 *           → Tap → Type content → Tap "Đăng"
 *
 * ⚠️ IMPORTANT:
 * - Accessibility Service must be enabled by user
 * - UI elements may change with app updates (Facebook, Messenger...)
 * - Use Vietnamese localized text for element matching
 * - Add delays between actions for UI to render
 */
class AppController(private val context: Context) {

    private val a11y get() = BizClawAccessibilityService

    // ─── Facebook ─────────────────────────────────────────────────

    /**
     * Post content to Facebook feed.
     *
     * Flow:
     * 1. Open Facebook app
     * 2. Find "Bạn đang nghĩ gì?" or "What's on your mind?"
     * 3. Tap to open composer
     * 4. Type content
     * 5. Tap "Đăng" / "Post"
     */
    suspend fun facebookPost(content: String): AutomationResult {
        if (!a11y.isRunning()) return AutomationResult.error("Accessibility service not enabled")

        return try {
            // Step 1: Open Facebook
            openApp("com.facebook.katana")
            delay(2000) // Wait for app to launch

            // Step 2: Find and tap the "What's on your mind?" field
            val tapped = a11y.clickByText("Bạn đang nghĩ gì")
                || a11y.clickByText("What's on your mind")
                || a11y.clickByText("Viết gì đó")
            if (!tapped) return AutomationResult.error("Cannot find post composer field")
            delay(1500)

            // Step 3: Type content
            val typed = a11y.typeText(content)
            if (!typed) return AutomationResult.error("Cannot type into post field")
            delay(500)

            // Step 4: Tap Post button
            val posted = a11y.clickByText("Đăng")
                || a11y.clickByText("Post")
            if (!posted) return AutomationResult.error("Cannot find Post button")

            AutomationResult.success("Posted to Facebook: ${content.take(50)}...")
        } catch (e: Exception) {
            AutomationResult.error("Facebook post failed: ${e.message}")
        }
    }

    /**
     * Comment on the first/current post visible on Facebook.
     */
    suspend fun facebookComment(comment: String): AutomationResult {
        if (!a11y.isRunning()) return AutomationResult.error("Accessibility service not enabled")

        return try {
            // Find and tap Comment button/icon
            val tapped = a11y.clickByText("Bình luận")
                || a11y.clickByText("Comment")
            if (!tapped) return AutomationResult.error("Cannot find Comment button")
            delay(1000)

            // Type comment
            val typed = a11y.typeText(comment)
            if (!typed) return AutomationResult.error("Cannot type comment")
            delay(300)

            // Send comment (Enter or send button)
            a11y.pressEnter()

            AutomationResult.success("Commented on Facebook: ${comment.take(50)}...")
        } catch (e: Exception) {
            AutomationResult.error("Facebook comment failed: ${e.message}")
        }
    }

    // ─── Messenger ────────────────────────────────────────────────

    /**
     * Reply to a Messenger conversation by contact name.
     *
     * Flow:
     * 1. Open Messenger
     * 2. Find conversation by name
     * 3. Tap to open
     * 4. Type and send message
     */
    suspend fun messengerReply(contactName: String, message: String): AutomationResult {
        if (!a11y.isRunning()) return AutomationResult.error("Accessibility service not enabled")

        return try {
            // Step 1: Open Messenger
            openApp("com.facebook.orca")
            delay(2000)

            // Step 2: Find and tap conversation
            val found = a11y.clickByText(contactName)
            if (!found) return AutomationResult.error("Cannot find conversation: $contactName")
            delay(1500)

            // Step 3: Find message input and type
            val typed = a11y.typeIntoField("Aa", message)
                || a11y.typeIntoField("Message", message)
                || a11y.typeIntoField("Nhắn tin", message)
                || a11y.typeText(message)
            if (!typed) return AutomationResult.error("Cannot type into message field")
            delay(300)

            // Step 4: Send (tap send button or press enter)
            val sent = a11y.clickByText("Gửi")
                || a11y.clickByText("Send")
                || a11y.pressEnter()
            if (!sent) return AutomationResult.error("Cannot send message")

            AutomationResult.success("Sent to $contactName: ${message.take(50)}...")
        } catch (e: Exception) {
            AutomationResult.error("Messenger reply failed: ${e.message}")
        }
    }

    /**
     * Read the last messages in the current Messenger conversation.
     */
    fun messengerReadMessages(): AutomationResult {
        if (!a11y.isRunning()) return AutomationResult.error("Accessibility service not enabled")

        val screen = a11y.readScreen() ?: return AutomationResult.error("Cannot read screen")

        if (!screen.packageName.contains("facebook.orca")) {
            return AutomationResult.error("Messenger is not open")
        }

        val messages = screen.elements
            .filter { it.text.isNotEmpty() && !it.isClickable && !it.isEditable }
            .map { it.text }
            .takeLast(10)

        return AutomationResult.success(
            "Messages:\n${messages.joinToString("\n")}"
        )
    }

    // ─── Zalo ─────────────────────────────────────────────────────

    /**
     * Send a Zalo message to a contact.
     */
    suspend fun zaloSendMessage(contactName: String, message: String): AutomationResult {
        if (!a11y.isRunning()) return AutomationResult.error("Accessibility service not enabled")

        return try {
            openApp("com.zing.zalo")
            delay(2000)

            val found = a11y.clickByText(contactName)
            if (!found) return AutomationResult.error("Cannot find: $contactName")
            delay(1500)

            val typed = a11y.typeIntoField("Nhắn tin", message)
                || a11y.typeIntoField("Tin nhắn", message)
                || a11y.typeText(message)
            if (!typed) return AutomationResult.error("Cannot type message")
            delay(300)

            a11y.clickByText("Gửi") || a11y.pressEnter()

            AutomationResult.success("Zalo sent to $contactName: ${message.take(50)}...")
        } catch (e: Exception) {
            AutomationResult.error("Zalo failed: ${e.message}")
        }
    }

    // ─── Generic App Control ──────────────────────────────────────

    /**
     * Read what's on the current screen (any app).
     */
    fun readCurrentScreen(): AutomationResult {
        if (!a11y.isRunning()) return AutomationResult.error("Accessibility service not enabled")

        val screen = a11y.readScreen() ?: return AutomationResult.error("Cannot read screen")

        val summary = buildString {
            appendLine("App: ${screen.packageName}")
            appendLine("Elements: ${screen.elementCount}")
            appendLine("---")
            for (element in screen.elements) {
                if (element.text.isNotEmpty()) {
                    val type = when {
                        element.isEditable -> "[INPUT]"
                        element.isClickable -> "[BUTTON]"
                        element.isScrollable -> "[SCROLL]"
                        else -> "[TEXT]"
                    }
                    appendLine("$type ${element.text}")
                }
            }
        }

        return AutomationResult.success(summary)
    }

    /**
     * Click any button/element by its text on the current screen.
     */
    fun clickElement(text: String): AutomationResult {
        if (!a11y.isRunning()) return AutomationResult.error("Accessibility service not enabled")

        val clicked = a11y.clickByText(text)
        return if (clicked) {
            AutomationResult.success("Clicked: $text")
        } else {
            AutomationResult.error("Element not found: $text")
        }
    }

    /**
     * Open an app by package name.
     */
    fun openApp(packageName: String) {
        val intent = context.packageManager.getLaunchIntentForPackage(packageName)
        if (intent != null) {
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            context.startActivity(intent)
        }
    }

    /**
     * Open a URL in the default browser.
     */
    fun openUrl(url: String) {
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        context.startActivity(intent)
    }
}

// ─── Result Type ──────────────────────────────────────────────────────

data class AutomationResult(
    val success: Boolean,
    val message: String,
) {
    companion object {
        fun success(message: String) = AutomationResult(true, message)
        fun error(message: String) = AutomationResult(false, message)
    }
}
