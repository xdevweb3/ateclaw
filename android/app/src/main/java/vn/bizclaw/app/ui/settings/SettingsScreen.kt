package vn.bizclaw.app.ui.settings

import android.content.Intent
import android.provider.Settings
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import vn.bizclaw.app.service.BizClawAccessibilityService
import vn.bizclaw.app.service.BizClawDaemonService

/**
 * Settings with Mode Selector:
 * - Mode A: LOCAL (Rust engine on phone, Ollama)
 * - Mode B: REMOTE (connect to VPS/Pi server)
 * - Mode C: HYBRID (local engine + remote agents)
 */
enum class RunMode(val label: String, val emoji: String, val description: String) {
    LOCAL("Độc lập", "📱", "Rust engine chạy trực tiếp trên phone. Dùng Ollama local, không cần internet."),
    REMOTE("Kết nối Server", "🌐", "Trỏ đến BizClaw VPS hoặc Raspberry Pi. Chat & điều khiển agent từ xa."),
    HYBRID("Kết hợp", "🔀", "Engine local + agent remote cùng lúc. Device tools local, LLM từ server."),
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    serverUrl: String,
    apiKey: String,
    isConnected: Boolean,
    onUpdateServer: (String, String) -> Unit,
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    var url by remember { mutableStateOf(serverUrl) }
    var key by remember { mutableStateOf(apiKey) }
    var showKey by remember { mutableStateOf(false) }
    var selectedMode by remember {
        mutableStateOf(
            if (serverUrl.contains("127.0.0.1") || serverUrl.contains("localhost"))
                RunMode.LOCAL
            else
                RunMode.REMOTE
        )
    }
    val isA11yEnabled = BizClawAccessibilityService.isRunning()
    val isDaemonRunning = BizClawDaemonService.isRunning()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Cài Đặt", fontWeight = FontWeight.Bold) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                    }
                },
            )
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            // ─── Mode Selector ────────────────────────────────────

            Text(
                "Chế độ hoạt động",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
            )

            RunMode.entries.forEach { mode ->
                val isSelected = selectedMode == mode
                OutlinedCard(
                    onClick = {
                        selectedMode = mode
                        when (mode) {
                            RunMode.LOCAL -> {
                                url = "http://127.0.0.1:3001"
                                onUpdateServer(url, key)
                            }
                            RunMode.REMOTE -> {} // User sets URL manually
                            RunMode.HYBRID -> {
                                // Start local daemon + keep remote URL
                                if (!isDaemonRunning) {
                                    BizClawDaemonService.start(context)
                                }
                            }
                        }
                    },
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.outlinedCardColors(
                        containerColor = if (isSelected)
                            MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.3f)
                        else
                            MaterialTheme.colorScheme.surface,
                    ),
                    border = CardDefaults.outlinedCardBorder().let {
                        if (isSelected) CardDefaults.outlinedCardBorder() else it
                    },
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        RadioButton(
                            selected = isSelected,
                            onClick = null,
                        )
                        Spacer(Modifier.width(12.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                "${mode.emoji} ${mode.label}",
                                fontWeight = FontWeight.SemiBold,
                            )
                            Text(
                                mode.description,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }

            HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))

            // ─── Connection Status ────────────────────────────────

            Card(
                colors = CardDefaults.cardColors(
                    containerColor = if (isConnected)
                        MaterialTheme.colorScheme.secondaryContainer
                    else
                        MaterialTheme.colorScheme.errorContainer,
                ),
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(
                        if (isConnected) Icons.Default.CheckCircle else Icons.Default.Error,
                        contentDescription = null,
                        tint = if (isConnected)
                            MaterialTheme.colorScheme.secondary
                        else
                            MaterialTheme.colorScheme.error,
                    )
                    Spacer(Modifier.width(12.dp))
                    Column {
                        Text(
                            if (isConnected) "Đã kết nối" else "Mất kết nối",
                            fontWeight = FontWeight.SemiBold,
                        )
                        Text(
                            when (selectedMode) {
                                RunMode.LOCAL -> "Engine local (127.0.0.1:3001)"
                                RunMode.REMOTE -> url
                                RunMode.HYBRID -> "Local + $url"
                            },
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }

            // ─── Server URL (Remote / Hybrid only) ────────────────

            if (selectedMode != RunMode.LOCAL) {
                Text("Máy Chủ", style = MaterialTheme.typography.titleMedium)

                OutlinedTextField(
                    value = url,
                    onValueChange = { url = it },
                    label = { Text("Server URL") },
                    placeholder = { Text("http://192.168.1.100:3001") },
                    modifier = Modifier.fillMaxWidth(),
                    leadingIcon = { Icon(Icons.Default.Dns, null) },
                    singleLine = true,
                )

                // API Key
                OutlinedTextField(
                    value = key,
                    onValueChange = { key = it },
                    label = { Text("API Key (optional)") },
                    placeholder = { Text("Pairing code từ server") },
                    modifier = Modifier.fillMaxWidth(),
                    leadingIcon = { Icon(Icons.Default.Key, null) },
                    singleLine = true,
                    visualTransformation = if (showKey)
                        VisualTransformation.None
                    else
                        PasswordVisualTransformation(),
                    trailingIcon = {
                        IconButton(onClick = { showKey = !showKey }) {
                            Icon(
                                if (showKey) Icons.Default.VisibilityOff
                                else Icons.Default.Visibility,
                                contentDescription = "Toggle",
                            )
                        }
                    },
                )

                // Save + Test
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    OutlinedButton(
                        onClick = { onUpdateServer(url, key) },
                        modifier = Modifier.weight(1f),
                    ) {
                        Icon(Icons.Default.NetworkCheck, null)
                        Spacer(Modifier.width(8.dp))
                        Text("Test")
                    }

                    Button(
                        onClick = {
                            onUpdateServer(url, key)
                            onBack()
                        },
                        modifier = Modifier.weight(1f),
                    ) {
                        Icon(Icons.Default.Save, null)
                        Spacer(Modifier.width(8.dp))
                        Text("Lưu")
                    }
                }

                // Preset servers
                Text(
                    "Máy Chủ Nhanh",
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                PresetServerButton("🌐 VPS (bizclaw.vn)", "https://apps.bizclaw.vn") { url = it }
                PresetServerButton("🍓 Raspberry Pi", "http://raspberrypi.local:3001") { url = it }
                PresetServerButton("🏠 LAN (192.168.x.x)", "http://192.168.1.100:3001") { url = it }
            }

            HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))

            // ─── Service Status ───────────────────────────────────

            Text(
                "Dịch vụ",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
            )

            // Daemon status
            ServiceStatusRow(
                emoji = if (isDaemonRunning) "🟢" else "🔴",
                label = "Agent Daemon",
                status = if (isDaemonRunning) "Đang chạy" else "Đã dừng",
                actionLabel = if (isDaemonRunning) "Dừng" else "Bật",
                onClick = {
                    if (isDaemonRunning) BizClawDaemonService.stop(context)
                    else BizClawDaemonService.start(context)
                },
            )

            // Accessibility status
            ServiceStatusRow(
                emoji = if (isA11yEnabled) "🟢" else "🟡",
                label = "Accessibility (App Control)",
                status = if (isA11yEnabled) "Đã bật" else "Chưa bật",
                actionLabel = if (isA11yEnabled) "Đã OK" else "Bật ngay",
                onClick = {
                    if (!isA11yEnabled) {
                        val intent = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
                        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                        context.startActivity(intent)
                    }
                },
                enabled = !isA11yEnabled,
            )

            HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))

            // ─── App Info ─────────────────────────────────────────

            Text("Thông tin", style = MaterialTheme.typography.titleMedium)
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f),
                ),
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    InfoItem("Phiên bản", "BizClaw Android v0.3.0")
                    InfoItem("Chế độ", selectedMode.label)
                    InfoItem("Rust FFI", "7 functions (bizclaw-ffi)")
                    InfoItem("Device Tools", "10 (battery, GPS, notification...)")
                    InfoItem("App Control", "Facebook, Messenger, Zalo, any app")
                }
            }
        }
    }
}

@Composable
fun ServiceStatusRow(
    emoji: String,
    label: String,
    status: String,
    actionLabel: String,
    onClick: () -> Unit,
    enabled: Boolean = true,
) {
    Card(
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
        ),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(emoji, fontSize = 20.sp)
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(label, fontWeight = FontWeight.Medium, style = MaterialTheme.typography.bodyMedium)
                Text(status, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            FilledTonalButton(
                onClick = onClick,
                enabled = enabled,
                shape = RoundedCornerShape(8.dp),
            ) {
                Text(actionLabel, style = MaterialTheme.typography.labelMedium)
            }
        }
    }
}

@Composable
fun InfoItem(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 3.dp),
    ) {
        Text(
            label,
            modifier = Modifier.width(110.dp),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            value,
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.Medium,
        )
    }
}

@Composable
fun PresetServerButton(
    label: String,
    serverUrl: String,
    onClick: (String) -> Unit,
) {
    OutlinedCard(
        onClick = { onClick(serverUrl) },
        modifier = Modifier.fillMaxWidth(),
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(label, modifier = Modifier.weight(1f))
            Text(
                serverUrl,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
