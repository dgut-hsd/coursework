"""
Task 4: Benchmark Report - Throughput & Latency Curves
Data source: lesson01.exe (debug mode)
"""

import matplotlib.pyplot as plt
import numpy as np

# ── Raw data ──
batch_sizes = np.array([1, 4, 16, 32, 64])
sentences_per_sec = np.array([4, 8, 11, 11, 12])
avg_lat_ms = np.array([269.36, 472.68, 1422.22, 2794.04, 5356.01])
p50_ms = np.array([266.12, 473.20, 1411.08, 2737.42, 5380.33])
p99_ms = np.array([386.54, 527.61, 1725.75, 3117.89, 5444.34])

# ── Create figure ──
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))
fig.suptitle(
    "EmbeddingEngine Benchmark (debug mode)",
    fontsize=14, fontweight="bold", y=1.02
)

# ── Subplot 1: Throughput ──
color_tp = "#2E86AB"
ax1.plot(batch_sizes, sentences_per_sec, "o-", color=color_tp,
         linewidth=2, markersize=8, markerfacecolor="white",
         markeredgewidth=2)
ax1.set_xlabel("Batch Size")
ax1.set_ylabel("Sentences / s")
ax1.set_title("Throughput vs Batch Size")
ax1.set_xticks(batch_sizes)
ax1.grid(True, linestyle="--", alpha=0.5)

# Annotate optimal batch size (knee point)
optimal_bs = 16
optimal_tp = sentences_per_sec[batch_sizes == optimal_bs][0]
ax1.annotate(
    f"Optimal BS = {optimal_bs}\n(knee point)",
    xy=(optimal_bs, optimal_tp),
    xytext=(optimal_bs + 12, optimal_tp + 1.5),
    fontsize=9,
    arrowprops=dict(arrowstyle="->", color="red", lw=1.5),
    bbox=dict(boxstyle="round,pad=0.3", fc="yellow", alpha=0.3),
)

# Throughput annotations
for i in range(len(batch_sizes)):
    ax1.text(
        batch_sizes[i] + 0.5, sentences_per_sec[i] + 0.2,
        f"{sentences_per_sec[i]:.0f}", fontsize=9,
        fontweight="bold", color=color_tp
    )

# ── Subplot 2: Latency ──
ax2.plot(batch_sizes, p50_ms, "s-", color="#A23B72", linewidth=2,
         markersize=8, markerfacecolor="white", markeredgewidth=2, label="P50 Latency")
ax2.plot(batch_sizes, p99_ms, "^-", color="#F18F01", linewidth=2,
         markersize=8, markerfacecolor="white", markeredgewidth=2, label="P99 Latency")
ax2.set_xlabel("Batch Size")
ax2.set_ylabel("Latency (ms)")
ax2.set_title("Latency vs Batch Size")
ax2.set_xticks(batch_sizes)
ax2.legend(fontsize=9)
ax2.grid(True, linestyle="--", alpha=0.5)

# P50 ~ P99 band
ax2.fill_between(batch_sizes, p50_ms, p99_ms, alpha=0.15,
                 color="#A23B72", label="P50 ~ P99 band")
ax2.legend(fontsize=9)

# ── Layout ──
plt.tight_layout()
plt.savefig("benchmark_report.png", dpi=200, bbox_inches="tight")
plt.show()

# ── Console output ──
print("=" * 55)
print("Benchmark Analysis")
print("=" * 55)
print(f"{'Batch':>6}  {'Sent/s':>8}  {'Avg Lat':>9}  {'P50':>9}  {'P99':>9}")
print("-" * 55)
for i in range(len(batch_sizes)):
    print(
        f"{batch_sizes[i]:>6}  {sentences_per_sec[i]:>8}  "
        f"{avg_lat_ms[i]:>9.2f}  {p50_ms[i]:>9.2f}  {p99_ms[i]:>9.2f}"
    )
print("-" * 55)

print(f"\n>> Optimal batch size: {optimal_bs}")
print(
    f"  Reasoning: throughput increased most from "
    f"{sentences_per_sec[batch_sizes == 4][0]} -> {optimal_tp}, "
    f"then flattened ({optimal_tp} -> "
    f"{sentences_per_sec[batch_sizes == 64][0]}) with linear latency growth."
)

print(f"\n>> P50 ~ P99 jitter")
for i in range(len(batch_sizes)):
    print(f"  batch={batch_sizes[i]:>2}: {p99_ms[i] - p50_ms[i]:>7.2f} ms")

print(f"\n>> Conclusion")
print(f"  - Throughput knee point at batch_size = {optimal_bs}")
print(f"  - Beyond {optimal_bs}, throughput no longer scales linearly")
print(f"  - Recommended batch_size for production: {optimal_bs}")
