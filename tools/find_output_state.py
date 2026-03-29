#!/usr/bin/env python3
"""
G6 Output State Discovery Script
Automatically tests all read commands to find which one contains the output state (headphones vs speakers)
"""

import hid
import time
import sys
from typing import Dict, List, Tuple

# G6 Device Info
VID = 0x041e
PID = 0x3256
INTERFACE = 4

# Known write commands from our g6_protocol.rs - FULL SEQUENCES!
# Output switching requires 30 commands (not just 2!)
def hex_to_bytes(hex_str):
    return bytes.fromhex(hex_str)

WRITE_HEADPHONES_CMDS = [
    hex_to_bytes("5a2c0500040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a2c0101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
]

WRITE_SPEAKERS_CMDS = [
    hex_to_bytes("5a2c0500020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a2c0101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
]

# Discovered read commands
READ_COMMANDS = {
    '0x05_status': bytes([0x5a, 0x05] + [0x00] * 62),
    '0x10': bytes([0x5a, 0x10] + [0x00] * 62),
    '0x20': bytes([0x5a, 0x20] + [0x00] * 62),
    '0x30': bytes([0x5a, 0x30] + [0x00] * 62),
    '0x15': bytes([0x5a, 0x15, 0x01] + [0x00] * 61),
    '0x3a_variant1': bytes([0x5a, 0x3a, 0x02, 0x09] + [0x00] * 60),
    '0x39': bytes([0x5a, 0x39, 0x01, 0x04] + [0x00] * 60),
    '0x3a_variant2': bytes([0x5a, 0x3a, 0x01, 0x07] + [0x00] * 60),
}


def find_g6_device():
    """Find and open the G6 device on the correct interface"""
    print(f"Searching for G6 device (VID: 0x{VID:04x}, PID: 0x{PID:04x}, Interface: {INTERFACE})...")
    
    for device_info in hid.enumerate():
        if (device_info['vendor_id'] == VID and 
            device_info['product_id'] == PID and 
            device_info['interface_number'] == INTERFACE):
            
            print(f"✓ Found G6 on interface {INTERFACE}")
            print(f"  Path: {device_info['path']}")
            
            device = hid.device()
            device.open_path(device_info['path'])
            return device
    
    print(f"✗ G6 device not found on interface {INTERFACE}")
    return None


def drain_buffer(device, max_reads=12):
    """Drain the response buffer after writes"""
    print("  Draining buffer...", end=" ")
    dummy_cmd = bytes([0x00]) + READ_COMMANDS['0x05_status']
    
    for i in range(max_reads):
        device.write(dummy_cmd)
        response = device.read(64)  # Blocking read
        
        # Check if we got a proper echo (5a 05)
        if len(response) > 1 and response[0] == 0x5a and response[1] == 0x05:
            print(f"drained after {i+1} reads ✓")
            return
    
    print(f"sent {max_reads} dummy reads ✓")


def set_output(device, mode: str):
    """Set output to headphones or speakers"""
    print(f"\nSetting output to {mode.upper()}...")
    
    if mode.lower() == 'headphones':
        commands = WRITE_HEADPHONES_CMDS
    else:  # speakers
        commands = WRITE_SPEAKERS_CMDS
    
    # Send all 30 commands (the full sequence)
    print(f"  Sending {len(commands)} commands...")
    for i, cmd in enumerate(commands):
        device.write(bytes([0x00]) + cmd)
        if (i + 1) % 10 == 0:
            print(f"  ✓ Sent {i + 1}/{len(commands)} commands")
    
    print(f"  ✓ All {len(commands)} commands sent")
    
    # Drain buffer
    drain_buffer(device)
    
    # CRITICAL: Wait for hardware relay to complete (solenoid click)
    # The physical switch takes ~2.5 seconds to fully settle
    print("  Waiting 3 seconds for hardware relay to settle...")
    print("  (You should hear a click from the device)")
    time.sleep(3.0)


def read_all_commands(device) -> Dict[str, bytes]:
    """Read all discovered commands and return responses"""
    print("\nReading all commands...")
    responses = {}
    
    # Drain buffer first
    drain_buffer(device)
    
    for cmd_name, cmd_bytes in READ_COMMANDS.items():
        # Send command
        device.write(bytes([0x00]) + cmd_bytes)
        
        # Read response
        response = device.read(64)  # Blocking read
        responses[cmd_name] = bytes(response)
        
        print(f"  {cmd_name:16s}: {' '.join(f'{b:02x}' for b in response[:16])}")
        
        # Small delay between commands
        time.sleep(0.05)
    
    return responses


def compare_responses(headphones: Dict[str, bytes], speakers: Dict[str, bytes]):
    """Compare responses and highlight differences"""
    print("\n" + "="*80)
    print("COMPARISON: Headphones vs Speakers")
    print("="*80)
    
    differences_found = False
    
    for cmd_name in READ_COMMANDS.keys():
        hp_resp = headphones[cmd_name]
        sp_resp = speakers[cmd_name]
        
        # Find differences
        diff_positions = []
        for i in range(min(len(hp_resp), len(sp_resp))):
            if hp_resp[i] != sp_resp[i]:
                diff_positions.append(i)
        
        if diff_positions:
            differences_found = True
            print(f"\n🔍 {cmd_name} - DIFFERENCES FOUND at positions: {diff_positions}")
            print(f"   Headphones: {' '.join(f'{b:02x}' for b in hp_resp[:32])}")
            print(f"   Speakers:   {' '.join(f'{b:02x}' for b in sp_resp[:32])}")
            
            # Highlight the specific differences
            for pos in diff_positions[:5]:  # Show first 5 differences
                print(f"   → Byte {pos}: 0x{hp_resp[pos]:02x} (HP) vs 0x{sp_resp[pos]:02x} (SP)")
        else:
            print(f"   {cmd_name:16s}: Identical")
    
    if not differences_found:
        print("\n⚠ No differences found in any command!")
    
    print("="*80)


def main():
    print("\n" + "="*80)
    print("G6 OUTPUT STATE DISCOVERY SCRIPT - AUTOMATED")
    print("="*80)
    print("\nPurpose: Find which byte represents output state (headphones vs speakers)")
    print("\nWhat this script does:")
    print("  1. Connect → Read state (CLEAN, before any writes)")
    print("  2. Write commands to switch output to HEADPHONES")
    print("  3. DISCONNECT + RECONNECT (clears buffer!)")
    print("  4. Read state again (CLEAN, fresh connection)")
    print("  5. Compare states to find which byte changed!")
    print("  6. Repeat for SPEAKERS to verify")
    print("\nTotal time: ~15-20 seconds")
    print("="*80)
    
    try:
        # BASELINE: Read state in current/unknown mode
        print("\n📡 STEP 1: Read BASELINE State")
        print("-"*80)
        print("Connecting to device...")
        device = find_g6_device()
        if not device:
            print("\n✗ Failed to connect")
            sys.exit(1)
        
        print("Reading current device state (before any writes)...")
        baseline_response = read_all_commands(device)
        device.close()
        print("✓ Baseline captured\n")
        time.sleep(0.5)
        
        # TEST 1: Switch to HEADPHONES then read
        print("\n📡 STEP 2: Test HEADPHONES Mode")
        print("-"*80)
        print("Reconnecting...")
        device = find_g6_device()
        if not device:
            print("\n✗ Failed to reconnect")
            sys.exit(1)
        
        print("Switching to HEADPHONES...")
        set_output(device, 'headphones')
        print("DISCONNECTING...")
        device.close()
        print("✓ Disconnected\n")
        
        print("Waiting 5 seconds for hardware AND firmware to fully settle...")
        time.sleep(5.0)
        
        print("RECONNECTING...")
        device = find_g6_device()
        if not device:
            print("\n✗ Failed to reconnect")
            sys.exit(1)
        
        print("Reading FRESH state (no writes since connect)...")
        headphones_response = read_all_commands(device)
        device.close()
        print("✓ Headphones state captured\n")
        time.sleep(0.5)
        
        # TEST 2: Switch to SPEAKERS then read
        print("\n📡 STEP 3: Test SPEAKERS Mode")
        print("-"*80)
        print("Reconnecting...")
        device = find_g6_device()
        if not device:
            print("\n✗ Failed to reconnect")
            sys.exit(1)
        
        print("Switching to SPEAKERS...")
        set_output(device, 'speakers')
        print("DISCONNECTING...")
        device.close()
        print("✓ Disconnected\n")
        
        print("Waiting 5 seconds for hardware AND firmware to fully settle...")
        time.sleep(5.0)
        
        print("RECONNECTING...")
        device = find_g6_device()
        if not device:
            print("\n✗ Failed to reconnect")
            sys.exit(1)
        
        print("Reading FRESH state (no writes since connect)...")
        speakers_response = read_all_commands(device)
        device.close()
        print("✓ Speakers state captured\n")
        
        # COMPARE ALL
        print("\n📊 STEP 4: Analyzing Differences")
        print("="*80)
        
        print("\n1️⃣ BASELINE vs HEADPHONES:")
        print("-"*80)
        compare_responses(baseline_response, headphones_response)
        
        print("\n2️⃣ HEADPHONES vs SPEAKERS:")
        print("-"*80)
        compare_responses(headphones_response, speakers_response)
        
        print("\n3️⃣ BASELINE vs SPEAKERS:")
        print("-"*80)
        compare_responses(baseline_response, speakers_response)
        
        print("\n" + "="*80)
        print("✓ DISCOVERY COMPLETE!")
        print("="*80)
        print("\n📋 RESULTS:")
        print("  Look at 'HEADPHONES vs SPEAKERS' comparison above")
        print("  The byte(s) that changed represent the output state!")
        print("\n  Use those byte positions to implement state parsing in")
        print("  rust/src-tauri/src/g6_device.rs")
        
    except Exception as e:
        print("\n" + "="*80)
        print("❌ ERROR OCCURRED")
        print("="*80)
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
    
    print("\n" + "="*80 + "\n")


if __name__ == '__main__':
    main()
