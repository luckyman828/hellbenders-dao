import { Fragment, useRef, useState, useEffect } from 'react';
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  ConfirmOptions,
  LAMPORTS_PER_SOL,
  SystemProgram,
  clusterApiUrl,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_CLOCK_PUBKEY
} from '@solana/web3.js'
import {AccountLayout,MintLayout,TOKEN_PROGRAM_ID,ASSOCIATED_TOKEN_PROGRAM_ID,Token} from "@solana/spl-token";
import useNotify from './notify'
import * as bs58 from 'bs58'
import * as anchor from "@project-serum/anchor";
import { programs } from '@metaplex/js';
import axios from "axios"
import {WalletConnect, WalletDisconnect} from '../wallet'
import { Container, Snackbar } from '@material-ui/core';
import Alert from '@material-ui/lab/Alert';
import { CircularProgress, Card, CardMedia, Grid, CardContent, Typography, BottomNavigation,
				Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper  } from '@mui/material'
import {createMint,createAssociatedTokenAccountInstruction,sendTransactionWithRetry} from './utility'
import { getOrCreateAssociatedTokenAccount } from '../helper/getOrCreateAssociatedTokenAccount'

let wallet : any
let conn = new Connection(clusterApiUrl("devnet"), "confirmed")
let notify: any

const { metadata: { Metadata } } = programs
const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")

// membership kind smart contract address and IDL
const SpawnNFTProgramId = new PublicKey('4b3b222CzcZKDcEjCSfxfANtBX1R7giLfmwLpoSu9HAC')
const SpawnNFTIdl = require('./hellbenders-spawn.json')
const SpawnNFTPOOL = new PublicKey('AvLbAGoVemaebNymNyL689Yp7m7FaJuMA9tRdKdZBZZ8')
const SpawnNFTSYMBOL = "SPAWN"


// membership kind smart contract address and IDL
const FakeIDNFTProgramId = new PublicKey('8S18mGzHyNGur85jAPoEjad8P8rywTpjyABbBEdmj2gb')
const FakeIDNFTIdl = require('./usdc-fake-id.json')
const FakeIDNFTPOOL = new PublicKey('6TVrWdVQAegLFUewKJLeZ7qsB43qXXwWxJmAu6ztsDmV')
const FakeIDNFTSYMBOL = "HELLPASS"

// ...  more nfts can be added here




// semi fungible token address and IDL



// ... more sfts can be added here


const confirmOption : ConfirmOptions = {commitment : 'finalized',preflightCommitment : 'finalized',skipPreflight : false}

interface Schedule{
	time : string;
	amount : string;
}

let defaultSchedule = {
	time : '', amount : ''
}

interface AlertState {
  open: boolean;
  message: string;
  severity: 'success' | 'info' | 'warning' | 'error' | undefined;
}

export default function Mint(){
	wallet = useWallet()
	notify = useNotify()

	const [pool, setPool] = useState<PublicKey>(FakeIDNFTPOOL)
	const [alertState, setAlertState] = useState<AlertState>({open: false,message: '',severity: undefined})
    const [isProcessing, setIsProcessing] = useState(false)
    const [holdingNfts, setHoldingNfts] = useState<any[]>([])
	const [poolData, setPoolData] = useState<any>(null)

	useEffect(()=>{
		getPoolData()
	},[pool])

	useEffect(()=>{
		if(poolData != null && wallet.publicKey != null){
			getNftsForOwner(FakeIDNFTProgramId, FakeIDNFTIdl, FakeIDNFTPOOL, FakeIDNFTSYMBOL, wallet.publicKey)
			// getNftsForOwner(wallet.publicKey, SYMBOL)
		}
	},[wallet.publicKey,poolData])

	const getTokenWallet = async (owner: PublicKey,mint: PublicKey) => {
	  return (
	    await PublicKey.findProgramAddress(
	      [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
	      ASSOCIATED_TOKEN_PROGRAM_ID
	    )
	  )[0];
	}

	const getMetadata = async (mint: PublicKey) => {
	  return (
	    await anchor.web3.PublicKey.findProgramAddress(
	      [
	        Buffer.from("metadata"),
	        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
	        mint.toBuffer(),
	      ],
	      TOKEN_METADATA_PROGRAM_ID
	    )
	  )[0];
	}

	const getEdition = async (mint: PublicKey) => {
	  return (
	    await anchor.web3.PublicKey.findProgramAddress(
	      [
	        Buffer.from("metadata"),
	        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
	        mint.toBuffer(),
	        Buffer.from("edition")
	      ],
	      TOKEN_METADATA_PROGRAM_ID
	    )
	  )[0];
	}

	const getPoolData = async() => {
		try{
			console.log(pool)
			const poolAddress = new PublicKey(pool)
			const randWallet = new anchor.Wallet(Keypair.generate())
			const provider = new anchor.Provider(conn, randWallet, confirmOption)
			const program = new anchor.Program(FakeIDNFTIdl, FakeIDNFTProgramId, provider)
			const pD = await program.account.pool.fetch(poolAddress)
    		setPoolData(pD)
		} catch(err){
			console.log(err)
			setPoolData(null)
		}
	}

	async function getNftsForOwner(contractAddress : PublicKey, contractIdl : any, collectionPool : PublicKey, symbol : string, owner : PublicKey) {
		console.log("symbol:", symbol);
		let allTokens: any[] = []
		const tokenAccounts = await conn.getParsedTokenAccountsByOwner(owner, {programId: TOKEN_PROGRAM_ID},"finalized");
		const randWallet = new anchor.Wallet(Keypair.generate())
		const provider = new anchor.Provider(conn,randWallet,confirmOption)
		console.log("contract");
		const program = new anchor.Program(contractIdl,contractAddress,provider)
		for (let index = 0; index < tokenAccounts.value.length; index++) {
			try{
				const tokenAccount = tokenAccounts.value[index];
				const tokenAmount = tokenAccount.account.data.parsed.info.tokenAmount;

				if (tokenAmount.amount == "1" && tokenAmount.decimals == "0") {
					let nftMint = new PublicKey(tokenAccount.account.data.parsed.info.mint)
					let pda = await getMetadata(nftMint)
					const accountInfo: any = await conn.getParsedAccountInfo(pda);
					let metadata : any = new Metadata(owner.toString(), accountInfo.value)
					console.log(metadata.data.data.symbol)
					if (metadata.data.data.symbol == symbol) {
						let [metadataExtended, bump] = await PublicKey.findProgramAddress([nftMint.toBuffer(), collectionPool.toBuffer()],contractAddress)

						if((await conn.getAccountInfo(metadataExtended)) == null) continue;
						let extendedData = await program.account.metadataExtended.fetch(metadataExtended)
						// let [parentMetadataExtended, bump2] = await PublicKey.findProgramAddress([extendedData.parentInvitation.toBuffer(), pool.toBuffer()],programId)
						// let parentExtendedData = await program.account.metadataExtended.fetch(parentMetadataExtended)
						
						// const { data }: any = await axios.get(metadata.data.data.uri)
						// const entireData = { ...data, id: Number(data.name.replace( /^\D+/g, '').split(' - ')[0])}

						allTokens.push({
							mint : nftMint, 
							metadata : pda, 
							tokenAccount :  tokenAccount.pubkey,
							metadataExtended : metadataExtended, 
							extendedData : extendedData,
							data : metadata.data.data, 
							// offChainData : entireData, 
							// parentId : parentExtendedData.number
						})
					}
				}
			} 
			catch(err) {
				continue;
			}
		}
		allTokens.sort(function(a:any, b: any){
			if(a.extendedData.number < b.extendedData.number) {return -1;}
			if(a.extendedData.number > b.extendedData.number) {return 1;}
			return 0;
		})
		console.log("all tokens:", allTokens)
		setHoldingNfts(allTokens)
		return allTokens
	}

	const mint = async() =>{
		try{
			// get provider from connection
			const provider = new anchor.Provider(conn, wallet as any, confirmOption)
			
			// get fake id nft program
			const fakeIDProgram = new anchor.Program(FakeIDNFTIdl,FakeIDNFTProgramId,provider)
			
			// get fake id nft pool
			const fakeIDPoolData = await fakeIDProgram.account.pool.fetch(FakeIDNFTPOOL)

			// get  hell dao nft program
			const spawnProgram = new anchor.Program(SpawnNFTIdl,SpawnNFTProgramId,provider)
			
			// get fake id nft pool
			const spawnPoolData = await spawnProgram.account.pool.fetch(SpawnNFTPOOL)
			
			// get config data of above pool
			const spawnConfigData = await spawnProgram.account.config.fetch(spawnPoolData.config)

			let transaction = new Transaction()
			let createTokenAccountTransaction = new Transaction()
			let instructions : TransactionInstruction[] = []
			let signers : Keypair[] = []
			const mintRent = await conn.getMinimumBalanceForRentExemption(MintLayout.span)
			const mintKey = createMint(instructions, wallet.publicKey,mintRent,0,wallet.publicKey,wallet.publicKey,signers)
			const recipientKey = await getTokenWallet(wallet.publicKey, mintKey)
			createAssociatedTokenAccountInstruction(instructions,recipientKey,wallet.publicKey,wallet.publicKey,mintKey)
			instructions.push(Token.createMintToInstruction(TOKEN_PROGRAM_ID,mintKey,recipientKey,wallet.publicKey,[],1))
			instructions.forEach(item=>transaction.add(item))
			const metadata = await getMetadata(mintKey)
			const masterEdition = await getEdition(mintKey)
			const [metadataExtended, bump] = await PublicKey.findProgramAddress([mintKey.toBuffer(),SpawnNFTPOOL.toBuffer()], SpawnNFTProgramId)
			let royaltyList : String[]= []

			// let formData = {
			// 	name : 'Hellbenders Dao or Die',
			// 	uri: `https://shdw-drive.genesysgo.net/7nPP797RprCMJaSXsyoTiFvMZVQ6y1dUgobvczdWGd35/clubhouse-wallet.json`,
			// }
			
			

			// creator
			const creatorMint = fakeIDPoolData.rootNft
			console.log(creatorMint.toString());
			const creatorResp = await conn.getTokenLargestAccounts(creatorMint,'finalized')
			console.log("creator response", creatorResp);
			if(creatorResp==null || creatorResp.value==null || creatorResp.value.length==0) throw new Error("Invalid creator")
			const creatorNftAccount = creatorResp.value[0].address
			const creatorInfo = await conn.getAccountInfo(creatorNftAccount,'finalized')
			if(creatorInfo == null) throw new Error('Creator NFT info failed')
			const accountCreatorInfo = AccountLayout.decode(creatorInfo.data)
			if(Number(accountCreatorInfo.amount)==0) throw new Error("Invalid Creator Info")
			const creatorWallet = new PublicKey(accountCreatorInfo.owner)

			// check if parent wallet is holding fake id nft
			const memberships = await getNftsForOwner(FakeIDNFTProgramId, FakeIDNFTIdl, FakeIDNFTPOOL, FakeIDNFTSYMBOL, wallet.publicKey);
			let parentMembership = memberships[0];

			// without fake ID case
			var parentMembershipAccount = creatorNftAccount;
			
			var parentMembershipOwner = creatorWallet;
			var grandParentMembershipOwner = creatorWallet;
			var grandGrandParentMembershipOwner = creatorWallet;
			var grandGrandGrandParentMembershipOwner = creatorWallet;
			var holdingFakeID = false;
			
			if (parentMembership) {
				
				// with fake ID case
				holdingFakeID = true;
				// parent 
				const parentMembershipResp = await conn.getTokenLargestAccounts(parentMembership.extendedData.parentNfp, 'finalized')
				
				if(parentMembershipResp!=null && parentMembershipResp.value != null && parentMembershipResp.value.length != 0) {

					parentMembershipAccount = parentMembershipResp.value[0].address
					let info = await conn.getAccountInfo(parentMembershipAccount, 'finalized')
					
					if(info != null) {

						let accountInfo = AccountLayout.decode(info.data)
						if(Number(accountInfo.amount) != 0) {

							parentMembershipOwner = new PublicKey(accountInfo.owner)
						}
						
					}
				}


				// grand parent 
				const grandParentMembershipResp = await conn.getTokenLargestAccounts(parentMembership.extendedData.grandParentNfp, 'finalized')
				if(grandParentMembershipResp!=null && grandParentMembershipResp.value != null && grandParentMembershipResp.value.length != 0) {

					const grandParentMembershipAccount = grandParentMembershipResp.value[0].address
					let info = await conn.getAccountInfo(grandParentMembershipAccount, 'finalized')
					if(info != null) {

						let accountInfo = AccountLayout.decode(info.data)
						if(Number(accountInfo.amount) != 0) {

							grandParentMembershipOwner = new PublicKey(accountInfo.owner)
						}
						
					}
				}


				// grand grand parent
				const grandGrandParentMembershipResp = await conn.getTokenLargestAccounts(parentMembership.extendedData.grandGrandParentNfp, 'finalized')
				if(grandGrandParentMembershipResp!=null && grandGrandParentMembershipResp.value != null && grandGrandParentMembershipResp.value.length != 0){
					const grandGrandParentMembershipAccount = grandGrandParentMembershipResp.value[0].address
					let info = await conn.getAccountInfo(grandGrandParentMembershipAccount, 'finalized')
					if(info != null) {

						let accountInfo = AccountLayout.decode(info.data)
						if(Number(accountInfo.amount) != 0) {
							grandGrandParentMembershipOwner = new PublicKey(accountInfo.owner)
						}
					}
				}


				const grandGrandGrandParentMembershipResp = await conn.getTokenLargestAccounts(parentMembership.extendedData.grandGrandGrandParentNfp, 'finalized')
				if(grandGrandGrandParentMembershipResp!=null && grandGrandGrandParentMembershipResp.value != null && grandGrandGrandParentMembershipResp.value.length != 0) {

					const grandGrandGrandParentMembershipAccount = grandGrandGrandParentMembershipResp.value[0].address

					let info = await conn.getAccountInfo(grandGrandGrandParentMembershipAccount, 'finalized')

					if(info != null) {

						let accountInfo = AccountLayout.decode(info.data)

						if(Number(accountInfo.amount) != 0) {
							grandGrandGrandParentMembershipOwner = new PublicKey(accountInfo.owner)
						} 
					}
				}
			} 	


			const legendaryToken = new PublicKey(spawnPoolData.legendary);
			var redlistTokenAccount = await getOrCreateAssociatedTokenAccount(
				conn,
				wallet.pubkey,
				legendaryToken,
				wallet.publicKey,
				wallet.signedTransaction
			);
			
			if(redlistTokenAccount[1]){

				const redlistGoldToken = new PublicKey(spawnPoolData.redlistGold);
				redlistTokenAccount = await getOrCreateAssociatedTokenAccount(
					conn,
					wallet.pubkey,
					redlistGoldToken,
					wallet.publicKey,
					wallet.signedTransaction
				);

				if(redlistTokenAccount[1]) {
					// if there is no red list gold token
					const redlistSteelToken = new PublicKey(spawnPoolData.redlistSteel);
					redlistTokenAccount = await getOrCreateAssociatedTokenAccount(
						conn,
						wallet.pubkey,
						redlistSteelToken,
						wallet.publicKey,
						wallet.signedTransaction
					);
					if(redlistTokenAccount[1]) {
						// if there is no redlist steel
						const redlistBlackToken = new PublicKey(spawnPoolData.redlistSteel);
						redlistTokenAccount = await getOrCreateAssociatedTokenAccount(
							conn,
							wallet.pubkey,
							redlistBlackToken,
							wallet.publicKey,
							wallet.signedTransaction
						);
					}
				}
			}

			

			if (redlistTokenAccount[1]) {
				// mint without redlist token
				transaction.add(spawnProgram.instruction.mint(
					new anchor.BN(bump),
					holdingFakeID,
					{
						accounts : {
							owner : wallet.publicKey,
							pool : SpawnNFTPOOL,
							config : spawnPoolData.config,
							nftMint : mintKey,
							nftAccount : recipientKey,
							metadata : metadata,
							masterEdition : masterEdition,
							metadataExtended : metadataExtended,
							// parentNftMint : parentMembership.extendedData.mint,
							parentNftAccount : parentMembershipAccount,
							parentNftOwner : parentMembershipOwner,
							
							// grandParentNftMint : parentMembership.extendedData.parentNfp,
							// grandParentNftAccount : grandParentMembershipAccount,
							grandParentNftOwner : grandParentMembershipOwner,
							
							// grandGrandParentNftMint : parentMembership.extendedData.grandParentNfp,
							// grandGrandParentNftAccount : grandGrandParentMembershipAccount,
							grandGrandParentNftOwner : grandGrandParentMembershipOwner,
							// grandGrandGrandParentNftMint : parentMembership.extendedData.grandGrandParentNfp,
							// grandGrandGrandParentNftAccount : grandGrandGrandParentMembershipAccount,
							grandGrandGrandParentNftOwner : grandGrandGrandParentMembershipOwner,
							
							scobyWallet : spawnPoolData.scobyWallet,
							creatorNftAccount : creatorNftAccount,
							creatorWallet : creatorWallet,
							// creatorScoutNftAccount : creatorScoutNftAccount,
							// creatorScoutWallet : creatorScoutWallet,
							tokenProgram : TOKEN_PROGRAM_ID,
							tokenMetadataProgram : TOKEN_METADATA_PROGRAM_ID,
							systemProgram : SystemProgram.programId,
							rent : SYSVAR_RENT_PUBKEY,					
						}
					}
				))

			} else {
				// mint with redlist token

				transaction.add(spawnProgram.instruction.mintWithRedlist(
					new anchor.BN(bump),
					holdingFakeID,
					{
						accounts : {
							owner : wallet.publicKey,
							pool : SpawnNFTPOOL,
							config : spawnPoolData.config,
							nftMint : mintKey,
							nftAccount : recipientKey,
							metadata : metadata,
							masterEdition : masterEdition,
							metadataExtended : metadataExtended,
							// parentNftMint : parentMembership.extendedData.mint,
							parentNftAccount : parentMembershipAccount,
							parentNftOwner : parentMembershipOwner,
							
							// grandParentNftMint : parentMembership.extendedData.parentNfp,
							// grandParentNftAccount : grandParentMembershipAccount,
							grandParentNftOwner : grandParentMembershipOwner,
							
							// grandGrandParentNftMint : parentMembership.extendedData.grandParentNfp,
							// grandGrandParentNftAccount : grandGrandParentMembershipAccount,
							grandGrandParentNftOwner : grandGrandParentMembershipOwner,
							// grandGrandGrandParentNftMint : parentMembership.extendedData.grandGrandParentNfp,
							// grandGrandGrandParentNftAccount : grandGrandGrandParentMembershipAccount,
							grandGrandGrandParentNftOwner : grandGrandGrandParentMembershipOwner,
							
							scobyWallet : spawnPoolData.scobyWallet,
							creatorNftAccount : creatorNftAccount,
							creatorWallet : creatorWallet,
							redlistTokenAccount :redlistTokenAccount[0],
							// creatorScoutNftAccount : creatorScoutNftAccount,
							// creatorScoutWallet : creatorScoutWallet,
							tokenProgram : TOKEN_PROGRAM_ID,
							tokenMetadataProgram : TOKEN_METADATA_PROGRAM_ID,
							systemProgram : SystemProgram.programId,
							rent : SYSVAR_RENT_PUBKEY,					
						}
					}
				))
			}
			
			await sendTransaction(transaction,signers)
			setAlertState({open: true, message:"Congratulations! Succeeded!",severity:'success'})
			await getPoolData()
		}catch(err){
			console.log(err)
			setAlertState({open: true, message:"Failed! Please try again!",severity:'error'})
		}
	}

	async function sendTransaction(transaction : Transaction, signers : Keypair[]) {
		transaction.feePayer = wallet.publicKey
		transaction.recentBlockhash = (await conn.getRecentBlockhash('max')).blockhash;
		await transaction.setSigners(wallet.publicKey,...signers.map(s => s.publicKey));
		if(signers.length != 0) await transaction.partialSign(...signers)
		const signedTransaction = await wallet.signTransaction(transaction);
		let hash = await conn.sendRawTransaction(await signedTransaction.serialize());
		await conn.confirmTransaction(hash);
		return hash
	}

	return <>
		<main className='content'>
			<div className="card">
			{
				poolData != null && 
				<h6 className="card-title">Mint Hellbenders Dao or Die: {poolData.countMinting+ "  Fake IDs were minted"}</h6>
			}
				<form className="form">
					{
						(wallet && wallet.connected) &&
						<button type="button" disabled={isProcessing==true} className="form-btn" style={{"justifyContent" : "center"}} onClick={async ()=>{
							setIsProcessing(true)
							setAlertState({open: true, message:"Processing transaction",severity: "warning"})
							await mint()
							setIsProcessing(false)
						}}>
							{ isProcessing==true ? "Processing..." :"Mint" }
						</button>
					}
					<WalletConnect/>
				</form>
			</div>
			<Grid container spacing={1}>
			{
				holdingNfts.map((item, idx)=>{
					return <Grid item xs={2}>
						<Card key={idx} sx={{minWidth : 300}}>
							{/* <CardMedia component="img" height="200" image={item.offChainData.image} alt="green iguana"/> */}
							<CardContent>
								<Typography gutterBottom variant="h6" component="div">
								{item.data.name}
								</Typography>
								<Typography variant="body2" color="text.secondary">
								{"mint : " + item.extendedData.mint}
								</Typography>
								<Typography variant="body2" color="text.secondary">
								{"parent : " + item.extendedData.parentNfp}
								</Typography>
								<Typography variant="body2" color="text.secondary">
								{"grandparent : "+ item.extendedData.grandParentNfp}
								</Typography>
								<Typography variant="body2" color="text.secondary">
								{"grandparent : "+ item.extendedData.grandGrandParentNfp}
								</Typography>
								<Typography variant="body2" color="text.secondary">
								{"grandparent : "+ item.extendedData.grandGrandGrandParentNfp}
								</Typography>
								<Typography variant="body2" color="text.secondary">
								{"Followers : " + item.extendedData.childrenCount}
								</Typography>
							</CardContent>
						</Card>
					</Grid>
				})
			}
			</Grid>
			<Snackbar
        open={alertState.open}
        autoHideDuration={alertState.severity != 'warning' ? 6000 : 1000000}
        onClose={() => setAlertState({ ...alertState, open: false })}
      >
        <Alert
        	iconMapping={{warning : <CircularProgress size={24}/>}}
          onClose={() => setAlertState({ ...alertState, open: false })}
          severity={alertState.severity}
        >
          {alertState.message}
        </Alert>
      </Snackbar>
		</main>
	</>
}