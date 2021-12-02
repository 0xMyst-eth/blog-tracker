import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { assert, expect } from 'chai';
import { ListTracker } from '../target/types/list_tracker';
const BN = require('bn.js');

describe('list-tracker', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ListTracker as Program<ListTracker>;

  // it('Is initialized!', async () => {
  //   // Add your test here.
  //   const tx = await program.rpc.initialize({});
  //   console.log("Your transaction signature", tx);
  // });

  async function getAccountBalance(pubkey) {
    // 
    let account = await provider.connection.getAccountInfo(pubkey);
  }

  async function createUser(startBalance){
    let airdropAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;
    let newUser = anchor.web3.Keypair.generate();

    let tx = await provider.connection.requestAirdrop(newUser.publicKey,airdropAmount);
    await provider.connection.confirmTransaction(tx);

    let newWallet = new anchor.Wallet(newUser);
    let newProvider = new anchor.Provider(provider.connection, newWallet, provider.opts);

    return {
      key:
        newUser,
        newWallet ,
      provider: newProvider
    };

  }

  function createUsers(amount){
    let promises = [];
    for(let i =0; i<amount;i++){
      let newUser = createUser(10);
      promises.push(newUser);
    }
    return promises;
  }

  function programForUser(user){
    return new anchor.Program(program.idl,program.programId, user.provider);
  }

  async function createList(name, owner, capacity = 16){
    // find PDA
    const [listAccount, bump] = await anchor.web3.PublicKey.findProgramAddress(
      ['todolist', owner.key.publicKey.toBytes(), name.slice(0,32)],
        program.programId
    )

    // create program for user !!!!!! study
    const userProgram = programForUser(owner);

    // create list
    await userProgram.rpc.initialize(
      name,
      capacity,
      bump, {
        accounts: {
          listToInit: listAccount,
          listCreator: owner.key.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );

    /
    let list = await userProgram.account.list.fetch(listAccount); 
    return { publicKey: listAccount, data: list }

  };

  it('Create a new List', async ()=> {
    const testUser = await createUser(10);
    let list = await createList('First List', testUser);

    expect(list.data.listOwner.toString(), ' List owner is set ').equals(testUser.key.publicKey.toString());
    expect(list.data.name,' The name is set ').equals('First List');
    expect(list.data.items.length, ' the list is emptyu ').equals(0);

 });

 async function addItem({list, itemName, user, bounty}){
  const creatorProgram = programForUser(user);
  let itemAccount = anchor.web3.Keypair.generate();

  let tx = await creatorProgram.rpc.add(
      list.data.name,
      itemName,
      new BN(bounty),
       {
        accounts: {
          itemAccount: itemAccount.publicKey,
          itemCreator: user.key.publicKey,
          listAccount: list.publicKey,
          listOwner: list.data.listOwner,
          systemProgram: anchor.web3.SystemProgram.programId
        },
        signers: [
          user.key,
          itemAccount
        ]
    });

    let [listData, itemData] = await Promise.all ([
        creatorProgram.account.list.fetch(list.publicKey),
        creatorProgram.account.item.fetch(itemAccount.publicKey),
    ]); 

    return {
      list: {
        publicKey: list.publicKey,
        data: listData,
      },
      item: {
        publicKey: itemAccount.publicKey,
        data: itemData
      }
    
    }
 }

 it('Creates an item', async () => {
   const listCreator = await createUser(10);
   const itemCreator = await createUser(10);

   let list = await createList(
     'Second List',
     listCreator
    );
    let item = await addItem(
    {
        list, 
        itemName: 'Bike', 
        user: itemCreator, 
        bounty: 5 * anchor.web3.LAMPORTS_PER_SOL
    });

    // assert.equal(listData.data.name, 'Second List', 'the  list name is correct !')


    console.log('item name is: ' + item.item.data.name);
    assert.equal(item.list.publicKey, list.publicKey, 'The list key is correct !')
  });

    

 
 

});


