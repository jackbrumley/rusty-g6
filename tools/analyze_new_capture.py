#!/usr/bin/env python3
"""
Analyze the new packet capture with crystalizer toggling
Looking for read patterns that Creative uses
"""

try:
    from scapy.all import rdpcap, Raw
    from scapy.layers.usb import USBpcap
except ImportError:
    print("Installing scapy...")
    import subprocess
    subprocess.check_call(["pip", "install", "scapy"])
    from scapy.all import rdpcap, Raw
    from scapy.layers.usb import USBpcap

import sys

def analyze_capture(filename):
    print(f"Reading {filename}...")
    try:
        packets = rdpcap(filename)
    except Exception as e:
        print(f"Error reading file: {e}")
        return
    
    print(f"Total packets: {len(packets)}")
    
    # Look for G6 device (VID 0x041e, PID 0x3256)
    g6_packets = []
    
    for i, pkt in enumerate(packets):
        if Raw in pkt:
            data = bytes(pkt[Raw].load)
            # Look for packets with our VID/PID or interface 4
            if len(data) > 0:
                g6_packets.append((i, pkt, data))
    
    print(f"Found {len(g6_packets)} packets with data\n")
    
    print("="*80)
    print("ANALYZING PACKET PATTERNS")
    print("="*80)
    
    # Group by time windows to find toggling pattern
    print("\nLooking for HID packets (likely 64-byte transfers)...")
    hid_packets = []
    
    for i, pkt, data in g6_packets:
        if len(data) >= 8:  # HID data
            # Show first 16 bytes
            hex_str = ' '.join(f'{b:02x}' for b in data[:16])
            print(f"Packet {i}: {len(data)} bytes - {hex_str}")
            hid_packets.append((i, data))
            
            if len(hid_packets) >= 100:  # Limit output
                print(f"\n... (showing first 100, total {len(g6_packets)} packets)")
                break
    
    # Look for read patterns (0x5a prefix)
    print("\n" + "="*80)
    print("PACKETS STARTING WITH 0x5A (G6 command prefix)")
    print("="*80)
    
    cmd_0x5a = []
    for i, data in hid_packets:
        if data[0] == 0x5a:
            hex_str = ' '.join(f'{b:02x}' for b in data[:32])
            print(f"Packet {i}: {hex_str}")
            cmd_0x5a.append((i, data))
    
    print(f"\nFound {len(cmd_0x5a)} packets with 0x5a prefix")
    
    # Analyze command bytes
    if cmd_0x5a:
        print("\n" + "="*80)
        print("COMMAND ANALYSIS (byte 1 after 0x5a)")
        print("="*80)
        
        commands = {}
        for i, data in cmd_0x5a:
            if len(data) > 1:
                cmd = data[1]
                if cmd not in commands:
                    commands[cmd] = []
                commands[cmd].append((i, data))
        
        for cmd in sorted(commands.keys()):
            print(f"\nCommand 0x{cmd:02x}: {len(commands[cmd])} occurrences")
            # Show first few examples
            for i, data in commands[cmd][:3]:
                hex_str = ' '.join(f'{b:02x}' for b in data[:16])
                print(f"  Packet {i}: {hex_str}")

if __name__ == '__main__':
    analyze_capture('packet-capture/sound-blasterx-g6-get-device-status.pcapng')
