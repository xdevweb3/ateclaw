package vn.bizclaw.app.ui.dashboard

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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import vn.bizclaw.app.service.BizClawDaemonService
import vn.bizclaw.app.service.DeviceCapabilities

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DashboardScreen(
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    val capabilities = remember { DeviceCapabilities(context) }
    var isDaemonRunning by remember { mutableStateOf(BizClawDaemonService.isRunning()) }

    // Auto-refresh device info
    val battery = remember { capabilities.getBatteryInfo() }
    val storage = remember { capabilities.getStorageInfo() }
    val network = remember { capabilities.getNetworkInfo() }
    val device = remember { capabilities.getDeviceInfo() }
    val oemWarning = remember { capabilities.getOemBatteryKillerWarning() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Dashboard", fontWeight = FontWeight.Bold) },
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
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            // ─── Daemon Control ───────────────────────────────────────

            Card(
                colors = CardDefaults.cardColors(
                    containerColor = if (isDaemonRunning)
                        MaterialTheme.colorScheme.primaryContainer
                    else
                        MaterialTheme.colorScheme.surfaceVariant,
                ),
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(20.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        if (isDaemonRunning) "🤖" else "😴",
                        fontSize = 48.sp,
                    )
                    Spacer(Modifier.height(8.dp))
                    Text(
                        if (isDaemonRunning) "Agent đang chạy" else "Agent đã dừng",
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.Bold,
                    )
                    Spacer(Modifier.height(16.dp))

                    Button(
                        onClick = {
                            if (isDaemonRunning) {
                                BizClawDaemonService.stop(context)
                            } else {
                                BizClawDaemonService.start(context)
                            }
                            isDaemonRunning = !isDaemonRunning
                        },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = if (isDaemonRunning)
                                MaterialTheme.colorScheme.error
                            else
                                MaterialTheme.colorScheme.primary,
                        ),
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Icon(
                            if (isDaemonRunning) Icons.Default.Stop else Icons.Default.PlayArrow,
                            null,
                        )
                        Spacer(Modifier.width(8.dp))
                        Text(if (isDaemonRunning) "Dừng Agent" else "Khởi động Agent")
                    }
                }
            }

            // ─── OEM Warning ──────────────────────────────────────────

            if (oemWarning != null) {
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.tertiaryContainer,
                    ),
                ) {
                    Row(
                        modifier = Modifier.padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Icon(
                            Icons.Default.Warning,
                            null,
                            tint = MaterialTheme.colorScheme.tertiary,
                        )
                        Spacer(Modifier.width(12.dp))
                        Text(
                            oemWarning,
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
            }

            // ─── Device Stats Grid ────────────────────────────────────

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                StatCard(
                    modifier = Modifier.weight(1f),
                    icon = Icons.Default.BatteryChargingFull,
                    label = "Pin",
                    value = "${battery.level}%",
                    subtext = if (battery.isCharging) "Đang sạc" else "${battery.temperatureCelsius}°C",
                    color = when {
                        battery.level > 50 -> MaterialTheme.colorScheme.secondary
                        battery.level > 20 -> MaterialTheme.colorScheme.tertiary
                        else -> MaterialTheme.colorScheme.error
                    },
                )
                StatCard(
                    modifier = Modifier.weight(1f),
                    icon = Icons.Default.Storage,
                    label = "Bộ nhớ",
                    value = "${storage.usedPercent}%",
                    subtext = "%.1f GB trống".format(storage.freeGb),
                    color = if (storage.usedPercent < 80)
                        MaterialTheme.colorScheme.secondary
                    else
                        MaterialTheme.colorScheme.error,
                )
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                StatCard(
                    modifier = Modifier.weight(1f),
                    icon = Icons.Default.Wifi,
                    label = "Mạng",
                    value = network.type.uppercase(),
                    subtext = network.wifiSsid ?: if (network.isConnected) "Connected" else "Offline",
                    color = if (network.isConnected)
                        MaterialTheme.colorScheme.secondary
                    else
                        MaterialTheme.colorScheme.error,
                )
                StatCard(
                    modifier = Modifier.weight(1f),
                    icon = Icons.Default.Memory,
                    label = "CPU",
                    value = "${device.cpuCores} cores",
                    subtext = "${device.freeRamMb} MB RAM free",
                    color = MaterialTheme.colorScheme.primary,
                )
            }

            // ─── Device Info ──────────────────────────────────────────

            Card(
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
                ),
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        "Thiết bị",
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                    )
                    Spacer(Modifier.height(8.dp))
                    InfoRow("Hãng", device.manufacturer)
                    InfoRow("Model", device.model)
                    InfoRow("Android", "${device.androidVersion} (SDK ${device.sdkVersion})")
                    InfoRow("BizClaw", "v0.3.0")
                }
            }
        }
    }
}

@Composable
fun StatCard(
    modifier: Modifier = Modifier,
    icon: ImageVector,
    label: String,
    value: String,
    subtext: String,
    color: androidx.compose.ui.graphics.Color,
) {
    Card(
        modifier = modifier,
        shape = RoundedCornerShape(16.dp),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
        ) {
            Icon(
                icon,
                contentDescription = label,
                tint = color,
                modifier = Modifier.size(24.dp),
            )
            Spacer(Modifier.height(8.dp))
            Text(
                value,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                color = color,
            )
            Text(
                label,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                subtext,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
            )
        }
    }
}

@Composable
fun InfoRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
    ) {
        Text(
            label,
            modifier = Modifier.width(80.dp),
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
