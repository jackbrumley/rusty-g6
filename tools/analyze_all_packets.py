#!/usr/bin/env python3
"""
Analyze ALL packets to find the crystalizer toggle pattern
Don't filter by 0x5a - look at everything!
"""

try:
    from scapy.all import rdpcap, Raw
except ImportError:
    print("Installing scapy...")
    import subprocess
    subprocess.check_call(["pip", "install", "scapy"])
    from scapy.all import rdpcap, Raw

def analyze_capture(filename):
    print(f"Reading {filename}...")
    packets = rdpcap(filename)
    print(f"Total packets: {len(packets)}\n")
    
    print("="*80)
    print("ANALYZING ALL PACKETS (looking for click patterns)")
    print("="*80)
    
    # Collect ALL packets with data, show timing
    all_data = []
    for i, pkt in enumerate(packets):
        if Raw in pkt:
            data = bytes(pkt[Raw].load)
            if len(data) > 0 and len(data) <= 100:  # Reasonable size
                timestamp = float(pkt.time) if hasattr(pkt, 'time') else i
                all_data.append((i, timestamp, data))
    
    print(f"Found {len(all_data)} packets with data\n")
    
    # Look for packets in different size ranges
    by_size = {}
    for i, ts, data in all_data:
        size = len(data)
        if size not in by_size:
            by_size[size] = []
        by_size[size].append((i, ts, data))
    
    print("Packets by size:")
    for size in sorted(by_size.keys()):
        print(f"  {size} bytes: {len(by_size[size])} packets")
    
    # Focus on smaller packets (HID reports are usually 8-65 bytes)
    print("\n" + "="*80)
    print("SMALL PACKETS (8-65 bytes) - Likely HID/Control")
    print("="*80)
    
    small_packets = []
    for size in range(8, 66):
        if size in by_size:
            small_packets.extend(by_size[size])
    
    # Sort by time
    small_packets.sort(key=lambda x: x[1])
    
    # Show with timing to identify click patterns
    if len(small_packets) > 0:
        print(f"\nShowing {min(200, len(small_packets))} small packets with timing:\n")
        
        prev_time = small_packets[0][1]
        for i, (pkt_num, ts, data) in enumerate(small_packets[:200]):
            time_diff = ts - prev_time
            hex_str = ' '.join(f'{b:02x}' for b in data[:min(32, len(data))])
            
            # Mark significant time gaps (possible click boundaries)
            marker = " <-- GAP" if time_diff > 0.1 else ""
            
            print(f"Pkt {pkt_num:5d} (+{time_diff:6.3f}s): {len(data):2d}b - {hex_str}{marker}")
            prev_time = ts
    
    # Look for repeating patterns
    print("\n" + "="*80)
    print("LOOKING FOR REPEATING PATTERNS")
    print("="*80)
    
    # Group identical packets
    unique_packets = {}
    for i, ts, data in small_packets:
        key = data[:16]  # First 16 bytes as key
        if key not in unique_packets:
            unique_packets[key] = []
        unique_packets[key].append((i, ts))
    
    # Show packets that appear multiple times
    repeating = [(data, occurrences) for data, occurrences in unique_packets.items() if len(occurrences) > 1]
    repeating.sort(key=lambda x: len(x[1]), reverse=True)
    
    print(f"\nFound {len(repeating)} repeating patterns:\n")
    for data, occurrences in repeating[:20]:  # Top 20
        hex_str = ' '.join(f'{b:02x}' for b in data)
        print(f"{len(occurrences):4d}x: {hex_str}")
        if len(occurrences) <= 10:
            print(f"       Packets: {[pkt for pkt, ts in occurrences]}")

if __name__ == '__main__':
    analyze_capture('packet-capture/sound-blasterx-g6-get-device-status.pcapng')
