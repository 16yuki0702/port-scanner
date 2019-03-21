// use pnet::packet::tcp::TcpPacket;
use pnet::packet::{ tcp, ipv4, ip, ethernet, MutablePacket, Packet};
use pnet::packet::tcp::TcpOption;
use pnet::datalink;
use pnet::datalink::Channel;
use pnet::datalink::MacAddr;
use std::net::Ipv4Addr;
use std::env;

extern crate rand;
use rand::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Bad nunber of arguemnts");
        std::process::exit(1);
    }

    let myaddr: Ipv4Addr = "192.168.11.22".parse().unwrap();
    let addr: Ipv4Addr = args[1].parse().unwrap();
    let _method = &args[2];

    let interface_name = env::args().nth(3).unwrap();

    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(|iface| iface.name == interface_name)
        .next()
        .expect("Failed to get interface");
    
    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => {
            panic!("Failed to create datalink channel {}", e)
        }
    };

    // thread::spawn(move || {
    //     recieve_packets(rx).unwrap();
    // });


    // let mut tcp_header: [u8; 20] = [0u8; 20];
    // let mut tcp_packet = tcp::MutableTcpPacket::new(&mut tcp_header[..]).unwrap();
    // tcp_packet.set_source(33333);
    // tcp_packet.set_destination(5432);
    // // tcp_packet.set_sequence(0);
    // // tcp_packet.set_acknowledgement(0);
    // tcp_packet.set_data_offset(5);
    // // tcp_packet.set_reserved(0);
    // tcp_packet.set_flags(tcp::TcpFlags::SYN); //SYNパケット
    // // tcp_packet.set_window(0);
    // let checksum = tcp::ipv4_checksum(&tcp_packet.to_immutable(), &myaddr, &addr);
    // tcp_packet.set_checksum(checksum);
    // // tcp_packet.set_urgent_ptr(0);

    // let mut ip_header: [u8; 20] = [0u8; 20];
    // let mut ipv4_packet = ipv4::MutableIpv4Packet::new(&mut ip_header[..]).unwrap();
    // ipv4_packet.set_version(4);
    // ipv4_packet.set_header_length(5);
    // // ipv4_packet.set_dscp(0);
    // // ipv4_packet.set_ecn(0);
    // ipv4_packet.set_total_length(100); //?
    // // ipv4_packet.set_identification(0);
    // // ipv4_packet.set_flags(0);
    // // ipv4_packet.set_fragment_offset(0);
    // ipv4_packet.set_ttl(128);
    // ipv4_packet.set_next_level_protocol(ip::IpNextHeaderProtocols::Tcp);
    // // ipv4_packet.set_checksum(0);
    // ipv4_packet.set_source(myaddr);
    // ipv4_packet.set_destination(addr);

    // let tcp_mut = tcp_packet.packet_mut();
    // ipv4_packet.set_payload(tcp_mut);

    // let mut ethernet_buffer = [0u8; 42];
    // let mut ethernet_packet = ethernet::MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    // ethernet_packet.set_destination(datalink::MacAddr::new(0x84, 0x89, 0xad, 0x8d, 0x85, 0xf6));
    // ethernet_packet.set_source(interface.mac_address());
    // ethernet_packet.set_ethertype(ethernet::EtherTypes::Ipv4);
    // let ipv4_mut = ipv4_packet.packet_mut();
    // ethernet_packet.set_payload(ipv4_mut);

    if let Some(packet) = build_random_packet(&addr) {
        tx.send_to(&packet, None);
    }

    // パケット作って送信する。
    // 返事をまつ。
    // 返事がきたらポートの解放状況がわかる

    // 送信したパケットに対する返答という判断をどのように行うか？ => 相手から自分のアドレス、送信元ポートに届くもの。
    // どのポートに送ったものという判断は？ => タイムアウトをつけて同期にする or 相手の返答ポート
    // メインスレッド：パケットの送信。
    // サブスレッド：パケットの受信
}

// fn recieve_packets(rx: datalink::DataLinkReceiver) {

// }

fn build_random_packet(destination_ip: &Ipv4Addr) -> Option<[u8; 66]> {
    const ETHERNET_HEADER_LEN: usize = 14;
    const TCP_HEADER_LEN: usize = 32;
    const IPV4_HEADER_LEN: usize = 20;

    let mut tmp_packet = [0u8; ETHERNET_HEADER_LEN + IPV4_HEADER_LEN + TCP_HEADER_LEN];
    
    // Setup Ethernet Header
    {
        let mut eth_header = ethernet::MutableEthernetPacket::new(&mut tmp_packet[..ETHERNET_HEADER_LEN]).unwrap();

        eth_header.set_destination(MacAddr::new(0x84, 0x89, 0xad, 0x8d, 0x85, 0xf6));
        eth_header.set_source(MacAddr::new(0xf4,0x0f,0x24,0x27,0xdb,0x00));
        eth_header.set_ethertype(ethernet::EtherTypes::Ipv4);
    }

    // Setup IP header
    {
        let mut ip_header = ipv4::MutableIpv4Packet::new(&mut tmp_packet[ETHERNET_HEADER_LEN..(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN)]).unwrap();
        ip_header.set_header_length(69);
        ip_header.set_total_length(52);
        ip_header.set_fragment_offset(16384);
        ip_header.set_next_level_protocol(ip::IpNextHeaderProtocols::Tcp);
        ip_header.set_source(Ipv4Addr::new(192, 168, 11, 22));
        ip_header.set_destination(destination_ip.clone());
        ip_header.set_identification(rand::random::<u16>());
        ip_header.set_ttl(128);
        ip_header.set_version(4);
        ip_header.set_flags(ipv4::Ipv4Flags::DontFragment);

        let checksum = ipv4::checksum(&ip_header.to_immutable());
        ip_header.set_checksum(checksum);
    }

    // Setup TCP header
    {
        let mut tcp_header = tcp::MutableTcpPacket::new(&mut tmp_packet[(ETHERNET_HEADER_LEN + IPV4_HEADER_LEN)..]).unwrap();

        tcp_header.set_source(rand::random::<u16>());
        tcp_header.set_destination(80);

        tcp_header.set_flags(tcp::TcpFlags::SYN);
        tcp_header.set_window(64240);
        tcp_header.set_data_offset(8);
        tcp_header.set_urgent_ptr(0);
        tcp_header.set_sequence(rand::random::<u32>());

        tcp_header.set_options(&vec![tcp::TcpOption::wscale(8), TcpOption::sack_perm(), TcpOption::mss(1460), TcpOption::nop(), TcpOption::nop()]);

        let checksum = tcp::ipv4_checksum(&tcp_header.to_immutable(), &Ipv4Addr::new(192, 168, 11, 22), &destination_ip);
        tcp_header.set_checksum(checksum);
    }

    Some(tmp_packet)
}
