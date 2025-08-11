let userId = await this.bitwardenContainerService.keyService.stateService.getActiveUserIdFromStorage();
let masterKey = await new Promise(async r => (await this.bitwardenContainerService.keyService.masterPasswordService.masterKey$(userId)).subscribe(v => r(v)));
let userKey = await this.bitwardenContainerService.keyService.masterPasswordService.decryptUserKeyWithMasterKey(masterKey, userId);
console.log("kmgr import", userId, userKey.keyB64);
